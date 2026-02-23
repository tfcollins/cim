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
import { SdkTaskProvider } from './providers/taskProvider';
import { SdkTasksViewProvider } from './providers/viewProvider';
import { TaskCommand } from './commands/taskCommand';

let outputChannel: vscode.OutputChannel;

export async function activate(context: vscode.ExtensionContext) {
    outputChannel = vscode.window.createOutputChannel('Code in Motion');
    outputChannel.appendLine('Code in Motion extension activated');

    const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
    if (!workspaceFolder) {
        outputChannel.appendLine('No workspace folder found');
        return;
    }

    outputChannel.appendLine(`Workspace folder: ${workspaceFolder.uri.fsPath}`);

    // Register task provider
    const taskProvider = new SdkTaskProvider(workspaceFolder, outputChannel);
    context.subscriptions.push(
        vscode.tasks.registerTaskProvider('sdk', taskProvider)
    );

    // Register view provider for sidebar
    const viewProvider = new SdkTasksViewProvider(workspaceFolder, outputChannel);
    const treeView = vscode.window.createTreeView('cim.tasksView', {
        treeDataProvider: viewProvider,
        showCollapseAll: true
    });
    context.subscriptions.push(treeView);

    // Register commands
    const taskCommand = new TaskCommand(workspaceFolder, outputChannel, viewProvider);

    context.subscriptions.push(
        vscode.commands.registerCommand(
            'cim.refreshTasks',
            () => taskCommand.refreshTasks()
        ),
        vscode.commands.registerCommand(
            'cim.runTask',
            (taskItem?: any) => taskCommand.runTask(taskItem?.target)
        ),
        vscode.commands.registerCommand(
            'cim.runBuild',
            () => taskCommand.runMakeTargetInTerminal('sdk-build')
        ),
        vscode.commands.registerCommand(
            'cim.runTest',
            () => taskCommand.runMakeTargetInTerminal('sdk-test')
        ),
        vscode.commands.registerCommand(
            'cim.runEnvsetup',
            () => taskCommand.runMakeTargetInTerminal('sdk-envsetup')
        ),
        vscode.commands.registerCommand(
            'cim.createSdkTerminal',
            () => taskCommand.getOrCreateSdkTerminal().show()
        ),
        vscode.commands.registerCommand(
            'cim.runClean',
            () => taskCommand.runMakeTargetInTerminal('sdk-clean')
        ),
        vscode.commands.registerCommand(
            'cim.runFlash',
            () => taskCommand.runMakeTargetInTerminal('sdk-flash')
        ),
        vscode.commands.registerCommand(
            'cim.editTasks',
            () => taskCommand.editTasks()
        )
    );

    outputChannel.appendLine('Code in Motion extension ready');
}

export function deactivate() {
    outputChannel.dispose();
}
