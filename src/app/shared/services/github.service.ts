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

import { Injectable, NgZone } from "@angular/core";
import { BehaviorSubject } from "rxjs";
import { GithubUser } from "../types";
import {
  TauriEventListener,
  TauriListenerEvent,
  TauriService,
} from "./tauri.service";

export type UsersMap = { [id: string]: GithubUser };

@Injectable({
  providedIn: "root",
})
export class GithubService implements TauriEventListener {
  private hasToken: boolean = false;
  private mainUser?: string;
  private users: UsersMap = {};

  private usersSubject: BehaviorSubject<UsersMap> =
    new BehaviorSubject<UsersMap>({});
  private availableSubject: BehaviorSubject<boolean> =
    new BehaviorSubject<boolean>(false);

  public constructor(private zone: NgZone, private tauriSvc: TauriService) {
    this.tauriSvc.register(TauriService.events.TOKEN_SET, this);
    this.tauriSvc.register(TauriService.events.USER_UPDATE, this);
    this.init();
  }

  public getListenerID(): string {
    return "gh-svc-listener";
  }

  public handleEvent(event: TauriListenerEvent): void {
    this.zone.run(() => {
      if (event.name === TauriService.events.TOKEN_SET) {
        this.hasToken = true;
        this.availableSubject.next(this.isAvailable());
      } else if (event.name === TauriService.events.USER_UPDATE) {
        let user = <GithubUser>event.payload;
        if (!this.mainUser) {
          this.mainUser = user.login;
        }
        this.users[user.login] = user;
        this.usersSubject.next(this.users);
      }
    });
  }

  private init(): void {
    this.tauriSvc.getToken().then((res: string) => {
      if (res !== "") {
        this.hasToken = true;
        this.availableSubject.next(this.isAvailable());
      }
    });
    this.tauriSvc
      .getUser()
      .then((res: GithubUser) => {
        this.users[res.login] = res;
        this.mainUser = res.login;
        this.usersSubject.next(this.users);
      })
      .catch(() => {});

    this.tauriSvc
      .getTrackedUsers()
      .then((res: GithubUser[]) => {
        res.forEach((user: GithubUser) => {
          this.users[user.login] = user;
        });
        this.usersSubject.next(this.users);
      })
      .catch(() => {
        console.error("error obtaining tracked users!");
      });
  }

  public isAvailable(): boolean {
    return this.hasToken;
  }

  public getMainUser(): string | undefined {
    return this.mainUser;
  }

  public getUsers(): BehaviorSubject<UsersMap> {
    return this.usersSubject;
  }

  public getAvailable(): BehaviorSubject<boolean> {
    return this.availableSubject;
  }
}
