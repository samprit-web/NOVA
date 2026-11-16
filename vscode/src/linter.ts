// NOVA Linter for VS Code

import * as vscode from 'vscode';
import { spawn } from 'child_process';

export class NovaLinter {
    private config: vscode.WorkspaceConfiguration;
    private diagnosticCollection: vscode.DiagnosticCollection;
    private runningProcesses: Map<string, ReturnType<typeof spawn>> = new Map();

    constructor(private context: vscode.ExtensionContext) {
        this.config = vscode.workspace.getConfiguration('nova');
        this.diagnosticCollection = vscode.languages.createDiagnosticCollection('nova');
        context.subscriptions.push(this.diagnosticCollection);
    }

    updateConfiguration() {
        this.config = vscode.workspace.getConfiguration('nova');
    }

    start() {
        // Lint on document change (debounced)
        let debounceTimer: NodeJS.Timeout | null = null;
        
        this.context.subscriptions.push(
            vscode.workspace.onDidChangeTextDocument(e => {
                if (e.document.languageId !== 'nova') return;
                
                if (debounceTimer) {
                    clearTimeout(debounceTimer);
                }
                
                debounceTimer = setTimeout(() => {
                    this.lintDocument(e.document);
                }, 500);
            })
        );

        // Lint on save
        this.context.subscriptions.push(
            vscode.workspace.onDidSaveTextDocument(document => {
                if (document.languageId === 'nova') {
                    this.lintDocument(document);
                }
            })
        );

        // Initial lint of open documents
        for (const doc of vscode.workspace.textDocuments) {
            if (doc.languageId === 'nova') {
                this.lintDocument(doc);
            }
        }
    }

    stop() {
        // Kill any running processes
        for (const [, process] of this.runningProcesses) {
            process.kill();
        }
        this.runningProcesses.clear();
        this.diagnosticCollection.clear();
    }

    async lintDocument(document: vscode.TextDocument) {
        if (!this.config.get('linter.enabled', true)) {
            this.diagnosticCollection.delete(document.uri);
            return;
        }

        const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
        if (!workspaceFolder) return;

        // Check if nova.toml exists
        const novaToml = vscode.Uri.joinPath(workspaceFolder.uri, 'nova.toml');
        try {
            await vscode.workspace.fs.stat(novaToml);
        } catch {
            // Not a NOVA project
            return;
        }

        // Run nova check
        await this.runNovaCheck(document, workspaceFolder.uri.fsPath);
    }

    private async runNovaCheck(document: vscode.TextDocument, cwd: string) {
        const key = document.uri.toString();
        
        // Kill any existing process for this document
        const existing = this.runningProcesses.get(key);
        if (existing) {
            existing.kill();
        }

        return new Promise<void>((resolve) => {
            const child = spawn('nova', ['check'], {
                cwd,
                stdio: ['ignore', 'pipe', 'pipe']
            });

            this.runningProcesses.set(key, child);

            let stdout = '';
            let stderr = '';

            child.stdout.on('data', (data) => {
                stdout += data.toString();
            });

            child.stderr.on('data', (data) => {
                stderr += data.toString();
            });

            child.on('close', (code) => {
                this.runningProcesses.delete(key);
                
                const diagnostics = this.parseOutput(stdout + stderr, document);
                this.diagnosticCollection.set(document.uri, diagnostics);
                resolve();
            });

            child.on('error', (err) => {
                this.runningProcesses.delete(key);
                console.error('Nova linter error:', err);
                resolve();
            });

            // Timeout after 30 seconds
            setTimeout(() => {
                if (this.runningProcesses.has(key)) {
                    child.kill();
                    this.runningProcesses.delete(key);
                    resolve();
                }
            }, 30000);
        });
    }

    private parseOutput(output: string, document: vscode.TextDocument): vscode.Diagnostic[] {
        const diagnostics: vscode.Diagnostic[] = [];
        
        // Parse nova check output format
        // Expected format: file:line:col: severity: message
        const lines = output.split('\n');
        
        for (const line of lines) {
            const match = line.match(/^(.+):(\d+):(\d+):\s*(error|warning|note|help):\s*(.+)$/);
            if (match) {
                const [, file, lineStr, colStr, severity, message] = match;
                
                // Only add diagnostics for the current document
                if (!file.endsWith(document.fileName.split('/').pop() || '')) {
                    continue;
                }

                const lineNum = parseInt(lineStr, 10) - 1;
                const colNum = parseInt(colStr, 10) - 1;
                
                if (lineNum < 0 || lineNum >= document.lineCount) continue;

                const range = new vscode.Range(lineNum, colNum, lineNum, colNum + 1);
                
                const diagnostic = new vscode.Diagnostic(
                    range,
                    message,
                    this.mapSeverity(severity)
                );
                
                diagnostic.source = 'nova';
                diagnostics.push(diagnostic);
            }
        }

        return diagnostics;
    }

    private mapSeverity(severity: string): vscode.DiagnosticSeverity {
        switch (severity.toLowerCase()) {
            case 'error':
                return vscode.DiagnosticSeverity.Error;
            case 'warning':
                return vscode.DiagnosticSeverity.Warning;
            case 'note':
                return vscode.DiagnosticSeverity.Information;
            case 'help':
                return vscode.DiagnosticSeverity.Hint;
            default:
                return vscode.DiagnosticSeverity.Error;
        }
    }
}