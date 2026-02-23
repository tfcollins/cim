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

export class SdkTaskProvider implements vscode.TaskProvider {
    private workspaceFolder: vscode.WorkspaceFolder;
    private outputChannel: vscode.OutputChannel;
    private cachedTasks: vscode.Task[] | undefined;

    constructor(workspaceFolder: vscode.WorkspaceFolder, outputChannel: vscode.OutputChannel) {
        this.workspaceFolder = workspaceFolder;
        this.outputChannel = outputChannel;
    }

    async provideTasks(token?: vscode.CancellationToken): Promise<vscode.Task[]> {
        return this.getTasks();
    }

    resolveTask(task: vscode.Task, token?: vscode.CancellationToken): vscode.Task | undefined {
        return task;
    }

    private getTasks(): vscode.Task[] {
        // If cached, return cache
        if (this.cachedTasks) {
            return this.cachedTasks;
        }

        const tasks: vscode.Task[] = [];
        const tasksJsonPath = path.join(this.workspaceFolder.uri.fsPath, '.vscode', 'tasks.json');

        try {
            if (fs.existsSync(tasksJsonPath)) {
                const tasksJsonContent = fs.readFileSync(tasksJsonPath, 'utf-8');
                const tasksJson = JSON.parse(tasksJsonContent);

                if (tasksJson.tasks && Array.isArray(tasksJson.tasks)) {
                    tasksJson.tasks.forEach((taskDef: any) => {
                        if (taskDef.type === 'sdk' || (taskDef.label && taskDef.label.startsWith('SDK:'))) {
                            const task = this.createTask(taskDef);
                            if (task) {
                                tasks.push(task);
                            }
                        }
                    });
                }

                this.outputChannel.appendLine(`Loaded ${tasks.length} SDK tasks from tasks.json`);
            } else {
                this.outputChannel.appendLine('No tasks.json found in workspace');
            }
        } catch (error) {
            this.outputChannel.appendLine(`Error loading tasks: ${error}`);
        }

        this.cachedTasks = tasks;
        return tasks;
    }

    private createTask(taskDef: any): vscode.Task | undefined {
        try {
            // Parse task definition from tasks.json
            const label = taskDef.label || 'SDK Task';
            const command = taskDef.command || 'make';
            const args = taskDef.args || [];
            const groupName = taskDef.group?.kind || 'build';
            const isDefault = taskDef.group?.isDefault || false;

            // Create execution
            const execution = new vscode.ShellExecution(command, args, {
                cwd: this.workspaceFolder.uri.fsPath
            });

            // Create task
            const task = new vscode.Task(
                { type: 'sdk', target: taskDef.target || label },
                this.workspaceFolder,
                label,
                'SDK Manager',
                execution
            );

            // Set group
            task.group = groupName === 'test' ? vscode.TaskGroup.Test : vscode.TaskGroup.Build;
            if (isDefault) {
                task.isDefault = true;
            }

            // Set presentation settings
            task.presentationOptions = {
                echo: true,
                reveal: vscode.TaskRevealKind.Always,
                focus: false,
                panel: vscode.TaskPanelKind.Shared,
                showReuseMessage: true,
                clear: false,
                ...taskDef.presentation
            };

            return task;
        } catch (error) {
            this.outputChannel.appendLine(`Error creating task: ${error}`);
            return undefined;
        }
    }

    public invalidateCache(): void {
        this.cachedTasks = undefined;
    }
}
