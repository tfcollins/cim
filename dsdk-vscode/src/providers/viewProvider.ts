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
import * as fs from 'fs';
import * as path from 'path';

export class SdkTasksViewProvider implements vscode.TreeDataProvider<SdkTaskItem> {
    private workspaceFolder: vscode.WorkspaceFolder;
    private outputChannel: vscode.OutputChannel;
    private _onDidChangeTreeData: vscode.EventEmitter<SdkTaskItem | undefined | null | void> =
        new vscode.EventEmitter<SdkTaskItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<SdkTaskItem | undefined | null | void> =
        this._onDidChangeTreeData.event;

    private tasks: SdkTaskItem[] = [];

    constructor(workspaceFolder: vscode.WorkspaceFolder, outputChannel: vscode.OutputChannel) {
        this.workspaceFolder = workspaceFolder;
        this.outputChannel = outputChannel;
        this.loadTasks();
    }

    getTreeItem(element: SdkTaskItem): vscode.TreeItem {
        this.outputChannel.appendLine(`getTreeItem called for: ${element.label}`);
        return element;
    }

    getChildren(element?: SdkTaskItem): Thenable<SdkTaskItem[]> {
        this.outputChannel.appendLine(`getChildren called, element: ${element ? element.label : 'root'}`);
        if (!element) {
            this.outputChannel.appendLine(`Returning ${this.tasks.length} root tasks`);
            return Promise.resolve(this.tasks);
        }
        return Promise.resolve([]);
    }

    private loadTasks(): void {
        this.tasks = [];
        const tasksJsonPath = path.join(this.workspaceFolder.uri.fsPath, '.vscode', 'tasks.json');

        try {
            if (fs.existsSync(tasksJsonPath)) {
                this.outputChannel.appendLine(`Reading tasks from: ${tasksJsonPath}`);
                const tasksJsonContent = fs.readFileSync(tasksJsonPath, 'utf-8');
                this.outputChannel.appendLine(`Tasks.json content length: ${tasksJsonContent.length} chars`);
                const tasksJson = JSON.parse(tasksJsonContent);

                if (tasksJson.tasks && Array.isArray(tasksJson.tasks)) {
                    this.outputChannel.appendLine(`Found ${tasksJson.tasks.length} total tasks in tasks.json`);
                    // Log first few characters of file to check if it's being read correctly
                    this.outputChannel.appendLine(`First 100 chars: ${tasksJsonContent.substring(0, 100)}...`);
                    
                    tasksJson.tasks.forEach((taskDef: any, index: number) => {
                        this.outputChannel.appendLine(`Task ${index}: ${taskDef.label}, type: ${taskDef.type}`);
                        
                        // Extract the actual make target from args
                        let makeTarget = 'unknown';
                        if (taskDef.args && Array.isArray(taskDef.args) && taskDef.args.length > 0) {
                            makeTarget = taskDef.args[0]; // First arg is the make target
                        }
                        
                        this.outputChannel.appendLine(`  -> Make target: ${makeTarget}`);
                        
                        const item = new SdkTaskItem(
                            taskDef.label || 'Untitled Task',
                            makeTarget,
                            vscode.TreeItemCollapsibleState.None
                        );

                        // Add command to run task when clicked
                        item.command = {
                            command: 'cim.runTask',
                            title: 'Run Task',
                            arguments: [item]
                        };

                        this.tasks.push(item);
                    });

                    this.outputChannel.appendLine(`Loaded ${this.tasks.length} tasks for sidebar view`);
                } else {
                    this.outputChannel.appendLine('No tasks array found in tasks.json');
                }
            }
        } catch (error) {
            this.outputChannel.appendLine(`Error loading tasks for view: ${error}`);
        }
    }

    private getTaskIcon(taskDef: any): vscode.ThemeIcon {
        const groupKind = taskDef.group?.kind || 'build';
        if (groupKind === 'test') {
            return new vscode.ThemeIcon('beaker');
        } else if (taskDef.label?.includes('clean') || taskDef.label?.includes('Clean')) {
            return new vscode.ThemeIcon('trash');
        } else if (taskDef.label?.includes('flash') || taskDef.label?.includes('Flash')) {
            return new vscode.ThemeIcon('zap');
        } else {
            return new vscode.ThemeIcon('wrench');
        }
    }

    public refresh(): void {
        this.outputChannel.appendLine('Refresh called - clearing cache and reloading tasks');
        this.tasks = []; // Clear cache first
        this.loadTasks();
        this.outputChannel.appendLine(`After refresh: ${this.tasks.length} tasks loaded`);
        this._onDidChangeTreeData.fire();
    }
}

export class SdkTaskItem extends vscode.TreeItem {
    public target: string;

    constructor(
        label: string,
        target: string,
        collapsibleState: vscode.TreeItemCollapsibleState = vscode.TreeItemCollapsibleState.None
    ) {
        super(label, collapsibleState);
        this.target = target;
        this.tooltip = `Run target: ${target}`;
        this.description = target;
        this.contextValue = 'sdkTask';
    }
}
