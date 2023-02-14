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
      .set_token(newToken)
      .then((res: boolean) => {
        this.tokenIsValidated = true;
        if (!res) {
          console.error("Unable to set api token!");
          this.errorSettingToken = true;
          this.successSettingToken = false;
          this.refreshToken();
        } else {
          this.successSettingToken = true;
          this.errorSettingToken = false;
        }
      })
      .catch((err) => {
        console.error("Error setting token: ", err);
        this.errorSettingToken = true;
        this.successSettingToken = false;
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
      .get_token()
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
    const match =
      (value.startsWith(GITHUB_PAT_PREFIX) ||
        value.startsWith(GITHUB_TKN_PREFIX)) &&
      isValidToken(value);
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
