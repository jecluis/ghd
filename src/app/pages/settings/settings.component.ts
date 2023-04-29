import { Component, OnInit } from "@angular/core";
import {
  AbstractControl,
  FormControl,
  ValidationErrors,
  ValidatorFn,
  Validators,
} from "@angular/forms";
import { invoke } from "@tauri-apps/api";
import { TauriService } from "src/app/shared/services/tauri.service";
import { GHDError } from "src/app/shared/types";

@Component({
  selector: "ghd-settings",
  templateUrl: "./settings.component.html",
  styleUrls: ["./settings.component.scss"],
})
export class SettingsComponent implements OnInit {
  public isLoading = true;
  public apiTokenFormControl: FormControl = new FormControl("", [
    Validators.required,
    githubTokenValidator(),
  ]);
  public errorSettingToken = false;
  public successSettingToken = false;
  public tokenIsValidated = false;
  public errorInvalidToken = false;

  private apiToken: string = "";

  public constructor(private tauriSvc: TauriService) {}

  public ngOnInit(): void {
    this.refreshToken();
  }

  public setToken() {
    if (!this.canSaveToken()) {
      return;
    }
    const newToken = this.apiTokenFormControl.value;
    this.tauriSvc
      .setToken(newToken)
      .then(() => {
        this.tokenIsValidated = true;
        this.successSettingToken = true;
        this.errorSettingToken = false;
      })
      .catch((err: number) => {
        console.error("Error setting token: ", err);
        this.errorSettingToken = true;
        this.successSettingToken = false;

        if (err === GHDError.BadTokenError) {
          this.errorInvalidToken = true;
        }
      });
  }

  public canSaveToken(): boolean {
    const newToken = this.apiTokenFormControl.value;
    return (
      this.apiTokenFormControl.valid &&
      newToken !== this.apiToken &&
      isValidToken(newToken)
    );
  }

  public changeToken() {
    console.debug("change token");
    this.tokenIsValidated = false;
  }

  public isTokenInvalid() {
    return (
      !this.apiTokenFormControl.valid ||
      (this.tokenIsValidated && this.errorSettingToken)
    );
  }

  private refreshToken() {
    this.tauriSvc
      .getToken()
      .then((res: string) => {
        this.apiToken = res;
        this.apiTokenFormControl.setValue(this.apiToken);
      })
      .finally(() => {
        this.isLoading = false;
      });
  }
}

const GITHUB_PAT_PREFIX = "github_pat_";
const GITHUB_TKN_PREFIX = "ghp_";

function githubTokenValidator(): ValidatorFn {
  return (control: AbstractControl): ValidationErrors | null => {
    const value: string = control.value;
    if (value.startsWith(GITHUB_PAT_PREFIX)) {
      return { patToken: { value: control.value } };
    }
    const match = value.startsWith(GITHUB_TKN_PREFIX) && isValidToken(value);
    return match ? null : { badToken: { value: control.value } };
  };
}

function isValidToken(value: string): boolean {
  let tkn = "";
  if (value.startsWith(GITHUB_PAT_PREFIX)) {
    tkn = value.substring(GITHUB_PAT_PREFIX.length);
  } else if (value.startsWith(GITHUB_TKN_PREFIX)) {
    tkn = value.substring(GITHUB_TKN_PREFIX.length);
  }
  return tkn.length > 0;
}
