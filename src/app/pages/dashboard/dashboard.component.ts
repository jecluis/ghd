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
  PREntry,
  PullRequestsService,
} from "src/app/shared/services/pull-requests.service";
import {
  TauriEventListener,
  TauriListenerEvent,
  TauriService,
} from "src/app/shared/services/tauri.service";
import { invoke } from "@tauri-apps/api";
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
  public iterationN = 0;
  public prs: PREntry[] = [];
  public user?: GithubUser;
  public trackedUsers: GithubUser[] = [];

  private prSubscription?: Subscription;
  private availSubscription?: Subscription;
  private usersSubscription?: Subscription;

  public constructor(
    private zone: NgZone,
    private modalSvc: NgbModal,
    private tauriSvc: TauriService,
    private prSvc: PullRequestsService,
    private ghSvc: GithubService,
  ) {}

  public ngOnInit(): void {
    this.tauriSvc.register(TauriService.events.ITERATION, this);
    this.prSubscription = this.prSvc.getPullRequests().subscribe({
      next: (entries: PREntry[]) => {
        this.prs = entries;
      },
    });

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
      },
    });
  }

  public ngOnDestroy(): void {
    this.tauriSvc.unregister(TauriService.events.ITERATION, this);
    if (!!this.prSubscription) {
      this.prSubscription.unsubscribe();
    }
    if (!!this.availSubscription) {
      this.availSubscription.unsubscribe();
    }
    if (!!this.usersSubscription) {
      this.usersSubscription.unsubscribe();
    }
  }

  public set iteration(value: number) {
    this.iterationN = value;
  }

  public get iteration(): number {
    return this.iterationN;
  }

  public getListenerID(): string {
    return "dashboard-evlistener";
  }

  public handleEvent(event: TauriListenerEvent): void {
    const evname = event.name;

    if (evname === TauriService.events.ITERATION) {
      this.handleIteration(<number>event.payload);
    } else if (evname === TauriService.events.PULL_REQUESTS_UPDATE) {
    }
  }

  public openTrackUserModal(): void {
    this.modalSvc.open(TrackUserModalComponent);
  }

  private handleIteration(n: number) {
    this.zone.run(() => {
      this.iterationN = n;
    });
  }
}
