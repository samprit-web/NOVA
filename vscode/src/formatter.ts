// NOVA Formatter for VS Code

import * as vscode from 'vscode';

export class NovaFormatter implements vscode.DocumentFormattingEditProvider, vscode.DocumentRangeFormattingEditProvider {
    private config: vscode.WorkspaceConfiguration;

    constructor(private context: vscode.ExtensionContext) {
        this.config = vscode.workspace.getConfiguration('nova');
    }

    updateConfiguration() {
        this.config = vscode.workspace.getConfiguration('nova');
    }

    provideDocumentFormattingEdits(
        document: vscode.TextDocument,
        options: vscode.FormattingOptions,
        token: vscode.CancellationToken
    ): vscode.ProviderResult<vscode.TextEdit[]> {
        if (!this.config.get('formatter.enabled', true)) {
            return [];
        }

        return this.formatDocument(document, options, new vscode.Range(0, 0, document.lineCount, 0));
    }

    provideDocumentRangeFormattingEdits(
        document: vscode.TextDocument,
        range: vscode.Range,
        options: vscode.FormattingOptions,
        token: vscode.CancellationToken
    ): vscode.ProviderResult<vscode.TextEdit[]> {
        if (!this.config.get('formatter.enabled', true)) {
            return [];
        }

        return this.formatDocument(document, options, range);
    }

    private formatDocument(
        document: vscode.TextDocument,
        options: vscode.FormattingOptions,
        range: vscode.Range
    ): vscode.TextEdit[] {
        const edits: vscode.TextEdit[] = [];
        const text = document.getText(range);
        const formatted = this.formatCode(text, options);

        if (formatted !== text) {
            edits.push(vscode.TextEdit.replace(range, formatted));
        }

        return edits;
    }

    private formatCode(code: string, options: vscode.FormattingOptions): string {
        // This is a simplified formatter
        // In a real implementation, this would call `nova fmt` or use a proper parser
        
        let result = code;
        
        // Ensure consistent indentation
        const indent = options.insertSpaces ? ' '.repeat(options.tabSize) : '\t';
        
        // Simple formatting rules
        const lines = result.split('\n');
        let indentLevel = 0;
        const formattedLines: string[] = [];

        for (let i = 0; i < lines.length; i++) {
            let line = lines[i].trimRight();
            const trimmed = line.trim();
            
            // Decrease indent for closing braces
            if (trimmed.startsWith('}') || trimmed.startsWith(']') || trimmed.startsWith(')')) {
                indentLevel = Math.max(0, indentLevel - 1);
            }
            
            // Apply current indentation
            if (trimmed.length > 0) {
                line = indent.repeat(indentLevel) + trimmed;
            } else {
                line = '';
            }
            
            formattedLines.push(line);
            
            // Increase indent for opening braces
            if (trimmed.endsWith('{') || trimmed.endsWith('[') || trimmed.endsWith('(')) {
                indentLevel++;
            }
            
            // Handle else, catch, etc.
            if (trimmed.startsWith('else') || trimmed.startsWith('catch') || trimmed.startsWith('finally')) {
                indentLevel = Math.max(0, indentLevel - 1);
            }
        }

        return formattedLines.join('\n');
    }
}