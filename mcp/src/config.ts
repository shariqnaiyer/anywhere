import fs from "node:fs";
import os from "node:os";
import path from "node:path";

export interface ResolvedConfig {
  baseUrl: string;
  token: string | null;
  /** Path we *would* have read from for the token (for error messages). */
  tokenPathChecked: string;
}

export interface ConfigOverrides {
  urlOverride?: string;
  tokenOverride?: string;
}

/** Mirror of Rust `dirs::config_dir()` for `things-api`. */
function configDir(): string {
  const home = os.homedir();
  if (process.platform === "darwin") {
    return path.join(home, "Library", "Application Support", "things-api");
  }
  // Linux / xdg
  const xdg = process.env.XDG_CONFIG_HOME;
  const base = xdg && xdg.length > 0 ? xdg : path.join(home, ".config");
  return path.join(base, "things-api");
}

function readTokenFile(): string | null {
  const p = path.join(configDir(), "auth_token");
  try {
    const raw = fs.readFileSync(p, "utf8").trim();
    return raw.length > 0 ? raw : null;
  } catch {
    return null;
  }
}

function tokenFilePath(): string {
  return path.join(configDir(), "auth_token");
}

export function resolveConfig(overrides: ConfigOverrides = {}): ResolvedConfig {
  // Base URL: --url → env → default
  let baseUrl =
    overrides.urlOverride ??
    process.env.THINGS_API_URL ??
    "http://127.0.0.1:3333";
  baseUrl = baseUrl.replace(/\/+$/, "");

  // Token: --token → env → file
  let token: string | null = null;
  if (overrides.tokenOverride && overrides.tokenOverride.length > 0) {
    token = overrides.tokenOverride;
  } else if (process.env.THINGS_AUTH_TOKEN && process.env.THINGS_AUTH_TOKEN.length > 0) {
    token = process.env.THINGS_AUTH_TOKEN;
  } else {
    token = readTokenFile();
  }

  return {
    baseUrl,
    token,
    tokenPathChecked: tokenFilePath(),
  };
}
