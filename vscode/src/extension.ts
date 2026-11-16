// NOVA Language Extension for VS Code

import * as vscode from 'vscode';
import { NovaLanguageClient } from './languageClient';
import { NovaFormatter } from './formatter';
import { NovaLinter } from './linter';

let languageClient: NovaLanguageClient;
let formatter: NovaFormatter;
let linter: NovaLinter;

export function activate(context: vscode.ExtensionContext) {
    console.log('NOVA Language extension is now active');

    // Initialize language client
    languageClient = new NovaLanguageClient(context);
    languageClient.start();

    // Initialize formatter
    formatter = new NovaFormatter(context);
    context.subscriptions.push(
        vscode.languages.registerDocumentFormattingEditProvider('nova', formatter)
    );
    context.subscriptions.push(
        vscode.languages.registerDocumentRangeFormattingEditProvider('nova', formatter)
    );

    // Initialize linter
    linter = new NovaLinter(context);
    linter.start();

    // Register commands
    registerCommands(context);

    // Register configuration listener
    context.subscriptions.push(
        vscode.workspace.onDidChangeConfiguration(e => {
            if (e.affectsConfiguration('nova')) {
                formatter.updateConfiguration();
                linter.updateConfiguration();
            }
        })
    );

    // Register document change listener for check-on-save
    context.subscriptions.push(
        vscode.workspace.onDidSaveTextDocument(document => {
            if (document.languageId === 'nova') {
                const config = vscode.workspace.getConfiguration('nova');
                if (config.get('checkOnSave')) {
                    runCheck(document.uri);
                }
                if (config.get('buildOnSave')) {
                    runBuild(document.uri);
                }
                if (config.get('testOnSave')) {
                    runTest(document.uri);
                }
            }
        })
    );
}

function registerCommands(context: vscode.ExtensionContext) {
    // Build command
    context.subscriptions.push(
        vscode.commands.registerCommand('nova.build', async (uri?: vscode.Uri) => {
            await runBuild(uri);
        })
    );

    // Run command
    context.subscriptions.push(
        vscode.commands.registerCommand('nova.run', async (uri?: vscode.Uri) => {
            await runRun(uri);
        })
    );

    // Test command
    context.subscriptions.push(
        vscode.commands.registerCommand('nova.test', async (uri?: vscode.Uri) => {
            await runTest(uri);
        })
    );

    // Check command
    context.subscriptions.push(
        vscode.commands.registerCommand('nova.check', async (uri?: vscode.Uri) => {
            await runCheck(uri);
        })
    );

    // Format command
    context.subscriptions.push(
        vscode.commands.registerCommand('nova.fmt', async (uri?: vscode.Uri) => {
            await runFormat(uri);
        })
    );

    // New project command
    context.subscriptions.push(
        vscode.commands.registerCommand('nova.new', async () => {
            await createNewProject();
        })
    );

    // Add dependency command
    context.subscriptions.push(
        vscode.commands.registerCommand('nova.add', async (uri?: vscode.Uri) => {
            await addDependency(uri);
        })
    );

    // Generate docs command
    context.subscriptions.push(
        vscode.commands.registerCommand('nova.doc', async (uri?: vscode.Uri) => {
            await generateDocs(uri);
        })
    );
}

async function runBuild(uri?: vscode.Uri) {
    const workspaceFolder = getWorkspaceFolder(uri);
    if (!workspaceFolder) {
        vscode.window.showErrorMessage('No workspace folder found');
        return;
    }

    const terminal = vscode.window.createTerminal({
        name: 'NOVA Build',
        cwd: workspaceFolder.uri.fsPath
    });
    terminal.show();
    terminal.sendText('nova build');
}

async function runRun(uri?: vscode.Uri) {
    const workspaceFolder = getWorkspaceFolder(uri);
    if (!workspaceFolder) {
        vscode.window.showErrorMessage('No workspace folder found');
        return;
    }

    const terminal = vscode.window.createTerminal({
        name: 'NOVA Run',
        cwd: workspaceFolder.uri.fsPath
    });
    terminal.show();
    terminal.sendText('nova run');
}

async function runTest(uri?: vscode.Uri) {
    const workspaceFolder = getWorkspaceFolder(uri);
    if (!workspaceFolder) {
        vscode.window.showErrorMessage('No workspace folder found');
        return;
    }

    const terminal = vscode.window.createTerminal({
        name: 'NOVA Test',
        cwd: workspaceFolder.uri.fsPath
    });
    terminal.show();
    terminal.sendText('nova test');
}

async function runCheck(uri?: vscode.Uri) {
    const workspaceFolder = getWorkspaceFolder(uri);
    if (!workspaceFolder) {
        vscode.window.showErrorMessage('No workspace folder found');
        return;
    }

    const terminal = vscode.window.createTerminal({
        name: 'NOVA Check',
        cwd: workspaceFolder.uri.fsPath
    });
    terminal.show();
    terminal.sendText('nova check');
}

async function runFormat(uri?: vscode.Uri) {
    const editor = vscode.window.activeTextEditor;
    if (!editor || editor.document.languageId !== 'nova') {
        vscode.window.showErrorMessage('No NOVA file open');
        return;
    }

    await vscode.commands.executeCommand('editor.action.formatDocument');
}

async function createNewProject() {
    const name = await vscode.window.showInputBox({
        prompt: 'Enter project name',
        placeHolder: 'my-nova-project',
        validateInput: value => {
            if (!value || !/^[a-zA-Z][a-zA-Z0-9_-]*$/.test(value)) {
                return 'Invalid project name. Use only letters, numbers, hyphens, and underscores.';
            }
            return null;
        }
    });

    if (!name) return;

    const template = await vscode.window.showQuickPick([
        { label: 'Binary (bin)', description: 'Executable application', detail: 'Creates a main.nova with fn main()' },
        { label: 'Library (lib)', description: 'Reusable library', detail: 'Creates lib.nova with public API' },
        { label: 'WASM', description: 'WebAssembly module', detail: 'Optimized for browser/edge deployment' },
        { label: 'Actor', description: 'Actor-based application', detail: 'With built-in actor system' }
    ], { placeHolder: 'Select project template' });

    if (!template) return;

    const folder = await vscode.window.showOpenDialog({
        canSelectFolders: true,
        canSelectFiles: false,
        canSelectMany: false,
        openLabel: 'Create Project Here'
    });

    if (!folder || folder.length === 0) return;

    const terminal = vscode.window.createTerminal({
        name: 'NOVA New',
        cwd: folder[0].fsPath
    });
    terminal.show();
    terminal.sendText(`nova new ${name} --template ${template.label.split(' ')[0].toLowerCase()}`);
}

async function addDependency(uri?: vscode.Uri) {
    const workspaceFolder = getWorkspaceFolder(uri);
    if (!workspaceFolder) {
        vscode.window.showErrorMessage('No workspace folder found');
        return;
    }

    const dep = await vscode.window.showInputBox({
        prompt: 'Enter dependency (name@version or name)',
        placeHolder: 'serde@1.0'
    });

    if (!dep) return;

    const features = await vscode.window.showInputBox({
        prompt: 'Optional features (comma-separated)',
        placeHolder: 'derive,std'
    });

    const terminal = vscode.window.createTerminal({
        name: 'NOVA Add',
        cwd: workspaceFolder.uri.fsPath
    });
    terminal.show();
    let cmd = `nova add ${dep}`;
    if (features) {
        cmd += ` --features ${features}`;
    }
    terminal.sendText(cmd);
}

async function generateDocs(uri?: vscode.Uri) {
    const workspaceFolder = getWorkspaceFolder(uri);
    if (!workspaceFolder) {
        vscode.window.showErrorMessage('No workspace folder found');
        return;
    }

    const terminal = vscode.window.createTerminal({
        name: 'NOVA Doc',
        cwd: workspaceFolder.uri.fsPath
    });
    terminal.show();
    terminal.sendText('nova doc --open');
}

function getWorkspaceFolder(uri?: vscode.Uri): vscode.WorkspaceFolder | undefined {
    if (uri) {
        return vscode.workspace.getWorkspaceFolder(uri);
    }
    const editor = vscode.window.activeTextEditor;
    if (editor) {
        return vscode.workspace.getWorkspaceFolder(editor.document.uri);
    }
    if (vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders.length > 0) {
        return vscode.workspace.workspaceFolders[0];
    }
    return undefined;
}

export function deactivate() {
    if (languageClient) {
        languageClient.stop();
    }
    if (linter) {
        linter.stop();
    }
}