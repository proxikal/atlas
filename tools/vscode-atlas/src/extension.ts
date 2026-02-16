import * as path from "path";
import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext) {
  const config = vscode.workspace.getConfiguration("atlas");
  const serverPath = config.get<string>("lsp.path", "atlas-lsp");
  const trace = config.get<string>("lsp.trace", "off");

  // Resolve the server binary; allow absolute, relative, or PATH lookup.
  const resolvedServerPath = path.isAbsolute(serverPath)
    ? serverPath
    : serverPath;

  const serverOptions: ServerOptions = {
    run: { command: resolvedServerPath, transport: TransportKind.stdio },
    debug: {
      command: resolvedServerPath,
      transport: TransportKind.stdio,
      options: { env: { RUST_LOG: "debug" } },
    },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "atlas" }],
    outputChannelName: "Atlas LSP",
    initializationOptions: {},
    traceOutputChannel: trace !== "off" ? vscode.window.createOutputChannel("Atlas LSP Trace") : undefined,
  };

  client = new LanguageClient(
    "atlasLanguageServer",
    "Atlas Language Server",
    serverOptions,
    clientOptions
  );

  // Optional: respect user trace setting
  client.setTrace(trace === "verbose" ? 2 : trace === "messages" ? 1 : 0);

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
