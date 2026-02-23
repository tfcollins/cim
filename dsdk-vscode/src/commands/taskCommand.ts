// Copyright (c) 2026 Analog Devices, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { SdkTasksViewProvider } from '../providers/viewProvider';

export class TaskCommand {
    private workspaceFolder: vscode.WorkspaceFolder;
    private outputChannel: vscode.OutputChannel;
    private viewProvider: SdkTasksViewProvider;
    private sdkTerminal: vscode.Terminal | undefined;

    constructor(workspaceFolder: vscode.WorkspaceFolder, outputChannel: vscode.OutputChannel, viewProvider: SdkTasksViewProvider) {
        this.workspaceFolder = workspaceFolder;
        this.outputChannel = outputChannel;
        this.viewProvider = viewProvider;
    }

    public getOrCreateSdkTerminal(): vscode.Terminal {
        // Check if existing terminal is still alive
        if (this.sdkTerminal && this.sdkTerminal.exitStatus === undefined) {
            return this.sdkTerminal;
        }

        // Create new terminal
        this.sdkTerminal = vscode.window.createTerminal({
            name: 'SDK Manager',
            cwd: this.workspaceFolder.uri.fsPath
        });

        this.outputChannel.appendLine('Created new SDK Manager terminal');
        this.outputChannel.appendLine('Tip: Set up your environment (source .venv/bin/activate, export vars, etc.) once in this terminal');

        return this.sdkTerminal;
    }

    async refreshTasks(): Promise<void> {
        this.viewProvider.refresh();
        vscode.window.showInformationMessage('SDK tasks refreshed');
    }

    async runTask(targetName?: string): Promise<void> {
        if (targetName) {
            // Run specific target directly in persistent terminal
            await this.runMakeTargetInTerminal(targetName);
            return;
        }

        // Show picker for all tasks
        const tasks = await vscode.tasks.fetchTasks({ type: 'sdk' });

        if (tasks.length === 0) {
            vscode.window.showWarningMessage('No SDK tasks found');
            return;
        }

        // Create quick pick items
        const items = tasks.map(task => ({
            label: task.name,
            task: task
        }));

        const selected = await vscode.window.showQuickPick(items, {
            placeHolder: 'Select a task to run',
            matchOnDescription: true
        });

        if (selected && selected.task.definition.target) {
            // Run in persistent terminal instead of using task execution
            await this.runMakeTargetInTerminal(selected.task.definition.target);
        }
    }

    async runMakeTargetInTerminal(target: string): Promise<void> {
        this.outputChannel.appendLine(`Running make target in persistent terminal: ${target}`);
        
        const terminal = this.getOrCreateSdkTerminal();
        terminal.show();
        terminal.sendText(`make ${target}`);
    }

    async editTasks(): Promise<void> {
        const tasksJsonPath = path.join(this.workspaceFolder.uri.fsPath, '.vscode', 'tasks.json');

        if (!fs.existsSync(tasksJsonPath)) {
            vscode.window.showWarningMessage('No tasks.json file found. Run "cim makefile" first.');
            return;
        }

        const document = await vscode.workspace.openTextDocument(tasksJsonPath);
        await vscode.window.showTextDocument(document);
    }
}
