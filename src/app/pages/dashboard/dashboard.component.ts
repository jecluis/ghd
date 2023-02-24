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

import { Component, NgZone, OnDestroy, OnInit } from "@angular/core";
import { Subscription } from "rxjs";
import {
  TauriEventListener,
  TauriListenerEvent,
} from "src/app/shared/services/tauri.service";
import {
  GithubService,
  UsersMap,
} from "src/app/shared/services/github.service";
import { NgbModal } from "@ng-bootstrap/ng-bootstrap";
import { TrackUserModalComponent } from "./track-user-modal/track-user-modal.component";

type GithubUser = {
  id: number;
  login: string;
  name: string;
  avatar_url: string;
};

@Component({
  selector: "ghd-dashboard",
  templateUrl: "./dashboard.component.html",
  styleUrls: ["./dashboard.component.scss"],
})
export class DashboardComponent
  implements OnInit, OnDestroy, TauriEventListener
{
  public isAvailable = false;
  public user?: GithubUser;
  public trackedUsers: GithubUser[] = [];
  public selectedUser?: GithubUser;

  private availSubscription?: Subscription;
  private usersSubscription?: Subscription;

  public constructor(
    private modalSvc: NgbModal,
    private ghSvc: GithubService,
  ) {}

  public ngOnInit(): void {
    this.availSubscription = this.ghSvc.getAvailable().subscribe({
      next: (res: boolean) => {
        this.isAvailable = res;
      },
    });
    this.usersSubscription = this.ghSvc.getUsers().subscribe({
      next: (res: UsersMap) => {
        let mainUser = this.ghSvc.getMainUser();
        if (!mainUser) {
          return;
        }
        this.user = res[mainUser];
        let userlst: GithubUser[] = [];
        Object.keys(res).forEach((login: string) => {
          if (login === mainUser) {
            return;
          }
          userlst.push(res[login]);
        });
        this.trackedUsers = userlst;
        if (!this.selectedUser) {
          this.selectedUser = this.user;
          console.log("select default user: ", this.selectedUser);
        }
      },
    });
  }

  public ngOnDestroy(): void {
    if (!!this.availSubscription) {
      this.availSubscription.unsubscribe();
    }
    if (!!this.usersSubscription) {
      this.usersSubscription.unsubscribe();
    }
  }

  public getListenerID(): string {
    return "dashboard-evlistener";
  }

  public handleEvent(event: TauriListenerEvent): void {}

  public openTrackUserModal(): void {
    this.modalSvc.open(TrackUserModalComponent);
  }

  public selectUser(user: GithubUser | undefined): void {
    this.selectedUser = user;
  }

  public isSelected(user: GithubUser | undefined): boolean {
    if (!this.selectedUser) {
      return false;
    } else if (!user) {
      return false;
    }
    return this.selectedUser.login === user.login;
  }
}
