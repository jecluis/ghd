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

import { Component } from "@angular/core";
import {
  AbstractControl,
  FormControl,
  ValidationErrors,
  ValidatorFn,
  Validators,
} from "@angular/forms";
import { NgbActiveModal } from "@ng-bootstrap/ng-bootstrap";
import { TauriService } from "src/app/shared/services/tauri.service";
import { GithubUser } from "src/app/shared/types";

@Component({
  selector: "ghd-track-user-modal",
  templateUrl: "./track-user-modal.component.html",
  styleUrls: ["./track-user-modal.component.scss"],
})
export class TrackUserModalComponent {
  public isChecking = false;
  public isAdding = false;
  public usernameFormControl: FormControl = new FormControl("", [
    Validators.required,
    usernameValidator(),
  ]);
  public checkedUser = false;
  public userExists = false;
  public user?: GithubUser;
  public addedUser?: GithubUser;
  public errorAdding = false;

  public constructor(
    private activeModal: NgbActiveModal,
    private tauriSvc: TauriService,
  ) {}

  public close(): void {
    this.activeModal.dismiss();
  }

  public checkUserExists(): void {
    this.isChecking = true;
    this.usernameFormControl.disable();

    const username = this.usernameFormControl.value;
    this.tauriSvc
      .checkUserExists(username)
      .then((res: GithubUser) => {
        this.userExists = true;
        this.user = res;
      })
      .catch(() => {
        this.userExists = false;
      })
      .finally(() => {
        this.isChecking = false;
        this.checkedUser = true;
        this.usernameFormControl.enable();
      });
  }

  public touchUsername(): void {
    this.checkedUser = false;
    this.userExists = false;
    this.user = undefined;
    this.addedUser = undefined;
    this.errorAdding = false;
  }

  public isInvalid(): boolean {
    return !this.usernameFormControl.valid;
  }

  public isSet(): boolean {
    return this.usernameFormControl.value !== "";
  }

  public submit(): void {
    if (
      this.isAdding ||
      !this.checkedUser ||
      (this.checkedUser && !this.userExists)
    ) {
      return;
    }

    this.isAdding = true;
    const username = this.usernameFormControl.value;
    this.tauriSvc
      .addTrackedUser(username)
      .then((res: GithubUser) => {
        this.addedUser = res;
      })
      .catch(() => {
        this.errorAdding = true;
      })
      .finally(() => {
        this.isAdding = false;
      });
  }
}

function usernameValidator(): ValidatorFn {
  return (control: AbstractControl): ValidationErrors | null => {
    const value: string = control.value;
    const regex = /^[\p{L}\p{N}]+$/u;
    const res = value.match(regex);
    return !!res ? null : { badUsername: { value: control.value } };
  };
}
