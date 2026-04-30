package com.farnam.mhrvf
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Intent
import android.net.VpnService
import android.os.Build
import android.os.ParcelFileDescriptor
import android.util.Log
import androidx.core.app.NotificationCompat
import com.github.shadowsocks.bg.Tun2proxy
import java.util.concurrent.atomic.AtomicBoolean
/**
 * Foreground VpnService that:
 *   1. Runs the mhrv-f Rust proxy (HTTP + SOCKS5 on the configured listener).
 *   2. Establishes a VPN TUN interface capturing all device traffic.
 *   3. Spawns tun2proxy in a background thread — it reads IP packets from
 *      the TUN fd, runs a userspace TCP/IP stack, and funnels every TCP/UDP
 *      flow through our local SOCKS5. Without step 3 the TUN captures
 *      traffic but nothing reads it → DNS_PROBE_STARTED in Chrome (the
 *      symptom that bit us on the first run).
 *
 * Loop-avoidance note: our own proxy's OUTBOUND connections to
 * google_ip:443 would normally be re-captured by the TUN ("traffic goes in
 * circles"). We break the loop by excluding this app's UID from the VPN
 * via `addDisallowedApplication(packageName)`. Everything else on the
 * device still gets routed through us.
 */
class MhrvVpnService : VpnService() {
    private var tun: ParcelFileDescriptor? = null
    private var proxyHandle: Long = 0L
    private var tun2proxyThread: Thread? = null
    private val tun2proxyRunning = AtomicBoolean(false)
    // Idempotency guard. teardown() is reachable from three paths:
    //   1. ACTION_STOP onStartCommand branch (background thread)
    //   2. onDestroy() (main thread, fires whenever stopSelf resolves
    //      OR Android decides to kill the service)
    //   3. Android revoking the VPN profile out-of-band (also onDestroy)
    // Running the full native cleanup sequence twice races two threads
    // through Tun2proxy.stop(), fd.close(), Native.stopProxy() on state
    // that's already been nullified — the second pass was the
    // SIGSEGV-or-zombie source. This flag makes the second call a
    // no-op.
    private val tornDown = AtomicBoolean(false)
    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        Log.i(TAG, "onStartCommand action=${intent?.action ?: "<null>"} startId=$startId")
        return when (intent?.action) {
            ACTION_STOP -> {
                // Drop foreground FIRST — that's what makes the status-bar
                // key icon disappear and lets the user see "Stop worked"
                // even if the native teardown below takes a few seconds
                // (e.g. a dozen in-flight Apps Script requests stuck in
                // their 30s timeout). The service itself stays alive until
                // stopSelf + the background thread below finish.
                try { stopForeground(STOP_FOREGROUND_REMOVE) } catch (t: Throwable) {
                    Log.w(TAG, "stopForeground: ${t.message}")
                }
                // Teardown can block on native shutdown (rt.shutdown_timeout
                // is 5s max, plus 2s for the tun2proxy join). Do it off the
                // main thread so we don't ANR.
                Thread({
                    teardown()
                    stopSelf()
                    Log.i(TAG, "teardown done, service stopping")
                }, "mhrv-teardown").start()
                START_NOT_STICKY
            }
            else -> {
                startEverything()
                START_STICKY
            }
        }
    }
    private fun startEverything() {
        // 1) Seed native with our app's private dir and boot the proxy.
        Native.setDataDir(filesDir.absolutePath)
        val cfg = ConfigStore.load(this)
        // Android 8+ requires every service started via
        // `startForegroundService()` to call `startForeground()` within a
        // short window or the system crashes the app with
        // `ForegroundServiceDidNotStartInTimeException`. Every `stopSelf()`
        // path below MUST therefore happen after a `startForeground()`
        // call — otherwise the user-visible symptom is "the app crashes
        // the instant I tap Start".
        val notifSocks5Port = cfg.socks5Port ?: (cfg.listenPort + 1)
        startForeground(NOTIF_ID, buildNotif(cfg.listenPort, notifSocks5Port))
        // Deployment ID + auth key are required for apps_script and full
        // modes talk to Apps Script. Direct mode runs without Apps Script credentials;
        // serverless JSON validates its own endpoint fields.
        val configReady = when (cfg.mode) {
            Mode.DIRECT -> true
            Mode.SERVERLESS_JSON -> cfg.hasServerlessConfig
            Mode.APPS_SCRIPT, Mode.FULL -> cfg.hasDeploymentId && cfg.authKey.isNotBlank()
        }
        if (!configReady) {
            Log.e(TAG, "Config is incomplete for ${cfg.mode}")
            try { stopForeground(STOP_FOREGROUND_REMOVE) } catch (_: Throwable) {}
            stopSelf()
            return
        }
        // Defensive stop: if a previous startEverything left a handle behind
        // (e.g. the user tapped Start twice, or a Stop path errored out
        // mid-teardown), release it first. Without this, Native.startProxy
        // below binds a brand-new listener while the old one still holds
        // :listenPort → "Address already in use" from the Rust side and the
        // app looks stuck in a half-configured state.
        if (proxyHandle != 0L) {
            Log.w(TAG, "startEverything: stale proxyHandle=$proxyHandle; stopping old proxy first")
            try { Native.stopProxy(proxyHandle) } catch (_: Throwable) {}
            proxyHandle = 0L
        }
        proxyHandle = Native.startProxy(cfg.toJson())
        if (proxyHandle == 0L) {
            Log.e(TAG, "Native.startProxy returned 0 — see logcat tag mhrv_jni")
            try { stopForeground(STOP_FOREGROUND_REMOVE) } catch (_: Throwable) {}
            stopSelf()
            return
        }
        // PROXY_ONLY mode: the user wants just the 127.0.0.1 HTTP + SOCKS5
        // listeners up, with no VpnService / no TUN. Typical reasons:
        // another VPN app already owns the system VPN slot, the user
        // wants per-app opt-in via Wi-Fi proxy settings, or the device
        // is a sandboxed/rooted setup where VpnService is unwelcome.
        // We already called startForeground() at the top of this method,
        // which is all PROXY_ONLY needs for the listener thread to survive
        // backgrounding.
        if (cfg.connectionMode == ConnectionMode.PROXY_ONLY) {
            Log.i(TAG, "PROXY_ONLY mode: listeners up, skipping VpnService/TUN")
            VpnState.setProxyHandle(proxyHandle)
            VpnState.setRunning(true)
            return
        }
        // 2) Establish the TUN. Key Builder calls:
        //    - addAddress(10.0.0.2/32): our local IP inside the tunnel.
        //    - addRoute(0.0.0.0/0): capture ALL IPv4 traffic. IPv6 isn't added,
        //      so v6 leaks stay up the normal route — fine for this app.
        //    - addDnsServer(1.1.1.1): DNS queries go to this IP, which ALSO
        //      hits our TUN — tun2proxy intercepts in Virtual DNS mode.
        //    - addDisallowedApplication(packageName) in ALL/EXCEPT modes:
        //      our OWN outbound connections bypass the TUN. Without this,
        //      the proxy's outbound to google_ip loops back through the TUN
        //      forever. In ONLY mode this is implicit because apps not added
        //      with addAllowedApplication() bypass the VPN, and Android does
        //      not allow mixing allowed and disallowed app lists.
        //    - setBlocking(false): we're going to hand the fd to tun2proxy,
        //      which does its own async I/O.
        val builder = Builder()
            .setSession(getString(R.string.app_name))
            .setMtu(MTU)
            .addAddress("10.0.0.2", 32)
            .addRoute("0.0.0.0", 0)
            .addDnsServer("1.1.1.1")
            .setBlocking(false)
        // Apply user-chosen app splitting on top of mandatory loop avoidance.
        //
        //   ALL    — no extra restriction; every other app routes through
        //            us. Matches pre-splitting behaviour.
        //   ONLY   — allow-list. addAllowedApplication() for each chosen
        //            package; anything missing from the list bypasses the
        //            VPN on the OS-native route. Self-exclusion is implicit
        //            as long as we don't add our own package to the allow-list.
        //   EXCEPT — deny-list. addDisallowedApplication() for each chosen
        //            package, additive with our self-exclude.
        //
        // Packages that are not installed (leftover selections from a
        // previous device) throw PackageManager.NameNotFoundException —
        // we log and skip rather than aborting the whole VPN start.
        when (cfg.splitMode) {
            SplitMode.ALL -> {
                disallowSelf(builder)
            }
            SplitMode.ONLY -> {
                if (cfg.splitApps.isEmpty()) {
                    Log.w(TAG, "ONLY mode with empty splitApps list — no app would get the VPN; falling back to ALL")
                    disallowSelf(builder)
                } else {
                    var allowed = 0
                    for (pkg in cfg.splitApps) {
                        if (pkg == packageName) continue
                        try {
                            builder.addAllowedApplication(pkg)
                            allowed++
                        } catch (e: Throwable) {
                            Log.w(TAG, "addAllowedApplication($pkg) failed: ${e.message}")
                        }
                    }
                    if (allowed == 0) {
                        Log.w(TAG, "ONLY mode had no usable apps — falling back to ALL")
                        disallowSelf(builder)
                    }
                }
            }
            SplitMode.EXCEPT -> {
                disallowSelf(builder)
                for (pkg in cfg.splitApps) {
                    if (pkg == packageName) continue  // already self-excluded above
                    try { builder.addDisallowedApplication(pkg) } catch (e: Throwable) {
                        Log.w(TAG, "addDisallowedApplication($pkg) failed: ${e.message}")
                    }
                }
            }
        }
        val parcelFd = try {
            builder.establish()
        } catch (t: Throwable) {
            Log.e(TAG, "VpnService.establish() failed: ${t.message}")
            null
        }
        if (parcelFd == null) {
            Log.e(TAG, "establish() returned null — is VPN permission granted?")
            Native.stopProxy(proxyHandle)
            proxyHandle = 0L
            try { stopForeground(STOP_FOREGROUND_REMOVE) } catch (_: Throwable) {}
            stopSelf()
            return
        }
        tun = parcelFd
        // 3) Start tun2proxy on a worker thread. It blocks until stop() or
        //    shutdown. We detach the fd so ownership transfers cleanly; the
        //    ParcelFileDescriptor (`tun`) still holds a reference, so closing
        //    it at teardown reliably tears down the TUN even if tun2proxy
        //    doesn't cleanly exit.
        val detachedFd = parcelFd.detachFd()
        tun2proxyRunning.set(true)
        // Full mode: enable udpgw so TUN-captured UDP (VoIP/STUN/etc.) can
        // multiplex through one persistent tunnel session. This magic address
        // stays near the end of 198.18.0.0/15 to avoid tun2proxy fake-DNS IPs
        // that commonly start near 198.18.0.1.
        val udpgwAddr = if (cfg.mode == Mode.FULL) "198.19.255.254:7300" else ""
        val worker = Thread({
            try {
                val rc = Tun2proxy.run(
                    "socks5://127.0.0.1:$socks5Port",
                    detachedFd,
                    /* closeFdOnDrop = */ true,
                    MTU.toChar(),
                    /* verbosity = info */ 3,
                    /* dnsStrategy = virtual */ 0,
                    udpgwAddr,
                )
                Log.i(TAG, "tun2proxy exited rc=$rc")
            } catch (t: Throwable) {
                Log.e(TAG, "tun2proxy crashed: ${t.message}", t)
            } finally {
                tun2proxyRunning.set(false)
            }
        }, "tun2proxy")
        try {
            worker.start()
            tun2proxyThread = worker
        } catch (t: Throwable) {
            Log.e(TAG, "tun2proxy thread start failed: ${t.message}", t)
            tun2proxyRunning.set(false)
            try {
                ParcelFileDescriptor.adoptFd(detachedFd).close()
            } catch (closeErr: Throwable) {
                Log.w(TAG, "adoptFd($detachedFd).close failed: ${closeErr.message}")
            }
            Native.stopProxy(proxyHandle)
            proxyHandle = 0L
            try { stopForeground(STOP_FOREGROUND_REMOVE) } catch (_: Throwable) {}
            stopSelf()
            return
        }
        // (startForeground was already called at the top of this method
        // to satisfy Android 8+'s foreground-service contract — see the
        // comment at the start of startEverything. Calling it here again
        // would be a no-op but wasteful.)
        // Publish "running" state for the UI's Connect/Disconnect button
        // to observe. Only flipped true once everything above succeeded —
        // if we'd flipped it earlier the button would light up green for
        // a failed-to-establish run.
        VpnState.setProxyHandle(proxyHandle)
        VpnState.setRunning(true)
    }
    /**
     * Tear down everything this service owns. Safe to call more than once:
     *   - `Tun2proxy.stop()` is idempotent on its side.
     *   - tun2proxyRunning gating means we skip the stop call when the
     *     worker thread has already exited.
     *   - `tun` and `proxyHandle` are nulled/zeroed after one pass, so a
     *     second call is a no-op.
     *
     * Shutdown order matters. Doing it wrong (we did originally) leaves
     * tun2proxy still forwarding packets into a half-dead Rust runtime
     * while the runtime is force-aborting its tasks — that's the scenario
     * that manifested as "Stop crashes the app" when there were in-flight
     * relay requests piled up against a dead Apps Script deployment. The
     * correct order is:
     *   1. Signal tun2proxy to stop (cooperative).
     *   2. Close the TUN fd — forces tun2proxy's read() to return EBADF.
     *   3. Join the tun2proxy thread (now it really will exit).
     *   4. Shut down the Rust proxy runtime (nothing left to forward to).
     */
    private fun teardown() {
        // Idempotency guard. Without this, onDestroy racing the
        // ACTION_STOP background thread has been observed to crash the
        // process — two threads into Tun2proxy.stop() and
        // Native.stopProxy(handle) where handle has already been zeroed
        // is a SIGSEGV waiting to happen. First caller wins, subsequent
        // callers return immediately.
        if (!tornDown.compareAndSet(false, true)) {
            Log.i(TAG, "teardown: already done, skipping (caller=${Thread.currentThread().name})")
            return
        }
        Log.i(
            TAG,
            "teardown: begin caller=${Thread.currentThread().name} " +
            "(tun2proxy running=${tun2proxyRunning.get()}, proxyHandle=$proxyHandle)",
        )
        // 1. Cooperative stop signal, bounded so a hung JNI call cannot
        //    stall the whole teardown thread.
        if (tun2proxyRunning.get()) {
            val stopper = Thread({
                try { Tun2proxy.stop() } catch (t: Throwable) {
                    Log.w(TAG, "Tun2proxy.stop: ${t.message}")
                }
            }, "mhrv-tun2proxy-stop").apply { start() }
            try { stopper.join(2_000) } catch (_: InterruptedException) {}
            if (stopper.isAlive) {
                Log.w(TAG, "Tun2proxy.stop did not return within 2s; proceeding")
            }
        }
        // 2. Close the TUN fd. Since we called detachFd earlier the
        //    ParcelFileDescriptor no longer owns the fd and close() here
        //    is a no-op; the real fd is owned by tun2proxy (closeFdOnDrop
        //    = true), which closes it on return from run().
        try { tun?.close() } catch (t: Throwable) {
            Log.w(TAG, "tun.close: ${t.message}")
        }
        tun = null
        // 3. Join the worker. 4s is enough in the happy case; if tun2proxy
        //    is stuck on something untoward we'd rather move on and force
        //    the runtime shutdown than hang forever.
        try {
            tun2proxyThread?.join(4_000)
        } catch (_: InterruptedException) {}
        val stillAlive = tun2proxyThread?.isAlive == true
        tun2proxyThread = null
        if (stillAlive) {
            Log.w(TAG, "tun2proxy thread still alive after join timeout — proceeding anyway")
        }
        // 4. Shut down the Rust proxy. Backed by `rt.shutdown_timeout(5s)`
        //    on the Rust side, so this is bounded even if the runtime
        //    has in-flight tasks (common when the Apps Script relay has
        //    piled up pending 30s timeouts).
        val handle = proxyHandle
        proxyHandle = 0L
        if (handle != 0L) {
            Log.i(TAG, "teardown: stopping proxy handle=$handle")
            try { Native.stopProxy(handle) } catch (t: Throwable) {
                Log.e(TAG, "Native.stopProxy threw: ${t.message}", t)
            }
        }
        // Flip UI state last — the button reverts to Connect only after
        // the native-side cleanup actually happened, not optimistically.
        VpnState.setProxyHandle(0L)
        VpnState.setRunning(false)
        Log.i(TAG, "teardown: done")
    }
    override fun onDestroy() {
        Log.i(TAG, "onDestroy entered")
        try {
            teardown()
        } catch (t: Throwable) {
            // Belt-and-suspenders. Crashing out of onDestroy takes the
            // whole process with it — user-visible as the app closing
            // right when they tap Stop, which is exactly the symptom we
            // are trying to fix. Anything that gets here is logged and
            // swallowed.
            Log.e(TAG, "onDestroy teardown threw: ${t.message}", t)
        }
        super.onDestroy()
        Log.i(TAG, "onDestroy done")
    }
    private fun disallowSelf(builder: Builder) {
        try {
            builder.addDisallowedApplication(packageName)
        } catch (e: Throwable) {
            // Shouldn't happen for our own package, but don't hard-fail.
            Log.w(TAG, "addDisallowedApplication failed: ${e.message}")
        }
    }
    private fun buildNotif(httpPort: Int, socks5Port: Int): Notification {
        val mgr = getSystemService(NotificationManager::class.java)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val ch = NotificationChannel(
                CHANNEL_ID,
                getString(R.string.notif_channel_name),
                NotificationManager.IMPORTANCE_LOW,
            ).apply {
                description = getString(R.string.notif_channel_desc)
                setShowBadge(false)
            }
            mgr.createNotificationChannel(ch)
        }
        val openIntent = PendingIntent.getActivity(
            this,
            0,
            Intent(this, MainActivity::class.java),
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
        )
        val stopIntent = PendingIntent.getService(
            this,
            1,
            Intent(this, MhrvVpnService::class.java).setAction(ACTION_STOP),
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
        )
        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle(getString(R.string.notif_vpn_title))
            .setContentText(getString(R.string.notif_vpn_text, httpPort, socks5Port))
            .setSmallIcon(android.R.drawable.presence_online)
            .setContentIntent(openIntent)
            .addAction(
                android.R.drawable.ic_menu_close_clear_cancel,
                getString(R.string.notif_action_stop),
                stopIntent,
            )
            .setOngoing(true)
            .setCategory(NotificationCompat.CATEGORY_SERVICE)
            .build()
    }
    companion object {
        private const val TAG = "MhrvVpnService"
        private const val CHANNEL_ID = "mhrv.vpn.status"
        private const val NOTIF_ID = 0x1001
        private const val MTU = 1500
        const val ACTION_STOP = "com.farnam.mhrvf.STOP"
    }
}
