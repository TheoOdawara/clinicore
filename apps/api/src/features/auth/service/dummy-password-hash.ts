import { hash } from "@node-rs/argon2";
import { randomBytes } from "node:crypto";

export const DUMMY_PASSWORD_HASH = Symbol("DUMMY_PASSWORD_HASH");

export function createDummyPasswordHash(): Promise<string> {
  return hash(randomBytes(32).toString("base64url"));
}
