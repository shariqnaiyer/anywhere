import SwiftUI
import AppKit

struct MenuView: View {
    @ObservedObject var serverManager: ServerManager
    @ObservedObject var accountStore: AccountStore
    @Binding var showSignup: Bool
    @Binding var showSettings: Bool

    @Environment(\.openWindow) private var openWindow

    var body: some View {
        // Status header (disabled; informational only).
        Text(statusLine)
            .disabled(true)

        Divider()

        if let url = effectiveURL {
            Button("Copy URL: \(url)") {
                copy(url)
            }
        } else if serverManager.state == .starting {
            Text("URL: getting tunnel…").disabled(true)
        } else {
            Text("URL: (none)").disabled(true)
        }

        if let token = accountStore.authToken {
            Button("Copy Token: \(token.prefix(16))…") {
                copy(token)
            }
        } else {
            Text("Token: (none — start server to generate)").disabled(true)
        }

        Divider()

        Button("Open Swagger UI") {
            openSwaggerUI()
        }
        .disabled(effectiveURL == nil)

        Button("Copy curl example") {
            copyCurlExample()
        }
        .disabled(accountStore.authToken == nil)

        Divider()

        // Server control
        switch serverManager.state {
        case .stopped:
            Button("Start server") { serverManager.start() }
        case .starting:
            Text("Starting…").disabled(true)
        case .running:
            Button("Stop server") { serverManager.stop() }
            Button("Restart server") { serverManager.restart() }
        case .error(let msg):
            Text("Error: \(msg)").disabled(true)
            Button("Restart server") { serverManager.restart() }
        }

        Button("Rotate token…") {
            do {
                _ = try accountStore.rotateToken()
                if case .running = serverManager.state {
                    serverManager.restart()
                }
            } catch {
                NSLog("rotate failed: \(error)")
            }
        }

        Divider()

        if accountStore.account == nil {
            Button("Sign up for a permanent URL…") {
                openWindow(id: "signup")
                NSApp.activate(ignoringOtherApps: true)
            }
        }

        Button("Settings…") {
            openWindow(id: "settings")
            NSApp.activate(ignoringOtherApps: true)
        }

        Divider()

        Button("Quit") {
            serverManager.stop()
            // Give SIGTERM a beat to land before exiting.
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.3) {
                NSApp.terminate(nil)
            }
        }
        .keyboardShortcut("q")
    }

    // MARK: - Effective URL

    /// The best available public URL: permanent account URL first, then the ephemeral
    /// trycloudflare.com URL the server prints when running without a signed-up account.
    private var effectiveURL: String? {
        accountStore.account?.url ?? serverManager.quickTunnelURL
    }

    // MARK: - Status line

    private var statusLine: String {
        let mode: String
        if let a = accountStore.account {
            mode = "named (\(a.username))"
        } else if serverManager.quickTunnelURL != nil {
            mode = "ephemeral (quick tunnel)"
        } else {
            mode = "ephemeral"
        }
        switch serverManager.state {
        case .stopped:
            return "● Stopped — \(mode)"
        case .starting:
            return "◐ Starting — \(mode)"
        case .running:
            return "● Online — \(mode)"
        case .error(let msg):
            return "✗ Error: \(msg)"
        }
    }

    // MARK: - Clipboard helpers

    private func copy(_ string: String) {
        let pb = NSPasteboard.general
        pb.clearContents()
        pb.setString(string, forType: .string)
    }

    private func copyCurlExample() {
        guard let token = accountStore.authToken else { return }
        let url = effectiveURL ?? "http://127.0.0.1:\(serverManager.port)"
        let curl = "curl -H 'Authorization: Bearer \(token)' \(url)/tasks"
        copy(curl)
    }

    private func openSwaggerUI() {
        let base = effectiveURL ?? "http://127.0.0.1:\(serverManager.port)"
        if let url = URL(string: "\(base)/swagger-ui") {
            NSWorkspace.shared.open(url)
        }
    }
}
