// Copyright 2023 Joao Eduardo Luis <joao@abysmo.io>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import { Injectable } from "@angular/core";
import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
import { getVersion as tauriGetVersion } from "@tauri-apps/api/app";
import { GithubUser, PullRequestEntry } from "../types";

export type TauriListenerEvent = {
  name: string;
  payload: any;
};

export interface TauriEventListener {
  handleEvent(event: TauriListenerEvent): void;
  getListenerID(): string;
}

@Injectable({
  providedIn: "root",
})
export class TauriService {
  public static events = {
    ITERATION: "iteration",
    USER_UPDATE: "user_update",
    TOKEN_SET: "token_set",
    USER_DATA_UPDATE: "user_data_update",
    TOKEN_INVALID: "token_invalid",
  };

  private listeners: Map<string, Map<string, TauriEventListener>>;

  public constructor() {
    this.listeners = new Map();
    this.init();
  }

  private init() {
    Object.values(TauriService.events).forEach((name: string) => {
      const evname: string = name;
      listen(evname, (event) => {
        const entry: TauriListenerEvent = {
          name: evname,
          payload: event.payload,
        };

        const listenerMap = this.listeners.get(evname);
        if (!listenerMap) {
          return;
        }
        for (const cb of listenerMap.values()) {
          cb.handleEvent(entry);
        }
      });

      this.listeners.set(evname, new Map());
    });
  }

  public register(evname: string, listener: TauriEventListener) {
    const listenerMap = this.listeners.get(evname);
    if (!listenerMap) {
      console.error(`Trying to listen for ${evname}, not tracked.`);
      return;
    }
    const listenerID = listener.getListenerID();
    listenerMap.set(listenerID, listener);
  }

  public unregister(evname: string, listener: TauriEventListener) {
    const listenerMap = this.listeners.get(evname);
    if (!listenerMap) {
      console.error(`Trying to unregister for ${evname}, not tracked.`);
      return;
    }
    const listenerID = listener.getListenerID();
    listenerMap.delete(listenerID);
  }

  public async getVersion(): Promise<string> {
    return await tauriGetVersion();
  }

  public getMainUser(): Promise<GithubUser> {
    return invoke("get_main_user");
  }

  public getToken(): Promise<string> {
    return invoke("get_token");
  }

  public setToken(token: string): Promise<boolean> {
    return invoke("set_token", { token: token });
  }

  public checkUserExists(username: string): Promise<GithubUser> {
    return invoke("check_user_exists", { username: username });
  }

  public addTrackedUser(username: string): Promise<GithubUser> {
    return invoke("add_tracked_user", { username: username });
  }

  public getTrackedUsers(): Promise<GithubUser[]> {
    return invoke("get_tracked_users");
  }

  public markPullRequestViewed(prid: number): Promise<void> {
    return invoke("pr_mark_viewed", { prid: prid });
  }

  public getPullRequestsByAuthor(login: string): Promise<PullRequestEntry[]> {
    return invoke("pr_get_list_by_author", { login: login });
  }

  public getInvolvedPullRequests(login: string): Promise<PullRequestEntry[]> {
    return invoke("pr_get_list_by_involved", { login: login });
  }

  public archiveIssue(issueId: number): Promise<void> {
    return invoke("archive_issue", { issueId: issueId });
  }
}
