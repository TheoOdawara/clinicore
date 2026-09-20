import { Injectable } from "@nestjs/common";
import { ConfigService } from "@nestjs/config";
import type { Environment } from "./env.validation";

@Injectable()
export class EnvironmentService {
  constructor(private readonly config: ConfigService<Environment, true>) {}

  get<Key extends keyof Environment>(key: Key): Environment[Key] {
    return this.config.getOrThrow(key, { infer: true });
  }
}
