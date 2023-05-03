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

import { OnInit } from "@angular/core";
import { Component, Input } from "@angular/core";
import { TrackedPRs } from "src/app/shared/types";

@Component({
  selector: "ghd-pull-requests-card",
  templateUrl: "./pull-requests-card.component.html",
  styleUrls: ["./pull-requests-card.component.scss"],
})
export class PullRequestsCardComponent implements OnInit {
  @Input()
  public prs: TrackedPRs = { toView: [], viewed: [], len: 0 };

  @Input()
  public login?: string;

  public isCollapsed: boolean = true;
  public hasLogin: boolean = false;

  public constructor() {}

  public ngOnInit(): void {
    this.hasLogin = !!this.login;
    if (!this.hasLogin) {
      console.error("Component missing required user login!");
    }
  }
}
