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
import { GHDError, GithubUser } from "../types";
import {
  TauriEventListener,
  TauriListenerEvent,
  TauriService,
} from "./tauri.service";

export type UsersMap = { [id: string]: GithubUser };
export type TokenStatus = {
  invalid: boolean;
  notSet: boolean;
};

@Injectable({
  providedIn: "root",
})
export class GithubService implements TauriEventListener {
  private tokenStatus: TokenStatus = { invalid: false, notSet: true };
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
        this.tokenStatus = { invalid: false, notSet: false };
        this.availableSubject.next(this.isAvailable());
      } else if (event.name === TauriService.events.USER_UPDATE) {
        let user = <GithubUser>event.payload;
        if (!this.mainUser) {
          this.mainUser = user.login;
        }
        this.users[user.login] = user;
        this.usersSubject.next(this.users);
      } else if (event.name == TauriService.events.TOKEN_INVALID) {
        this.tokenStatus = { invalid: true, notSet: false };
        this.availableSubject.next(this.isAvailable());
      }
    });
  }

  private init(): void {
    this.tauriSvc
      .getToken()
      .then((res: string) => {
        if (res !== "") {
          this.tokenStatus = { invalid: false, notSet: false };
          this.availableSubject.next(this.isAvailable());
        }
      })
      .catch((err: number) => {
        if (err === GHDError.BadTokenError) {
          this.tokenStatus = { invalid: true, notSet: false };
        } else if (err === GHDError.TokenNotFoundError) {
          this.tokenStatus = { invalid: false, notSet: true };
        }
        this.availableSubject.next(this.isAvailable());
      });
    this.tauriSvc
      .getMainUser()
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
    return !this.tokenStatus.notSet && !this.tokenStatus.invalid;
  }

  public hasTokenSet(): boolean {
    return !this.tokenStatus.notSet;
  }

  public isTokenInvalid(): boolean {
    return this.tokenStatus.invalid;
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
