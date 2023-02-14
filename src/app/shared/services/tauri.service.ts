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
import { GithubUser } from "../types";

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
    PULL_REQUESTS_UPDATE: "pull_requests_update",
    USER_UPDATE: "user_update",
    TOKEN_SET: "token_set",
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

  public get_user(): Promise<GithubUser> {
    return invoke("get_user");
  }

  public get_token(): Promise<string> {
    return invoke("get_token");
  }

  public set_token(token: string): Promise<boolean> {
    return invoke("set_token", { token: token });
  }
}
