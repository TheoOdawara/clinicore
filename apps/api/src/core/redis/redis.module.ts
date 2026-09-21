import { Global, Inject, Module, type OnModuleDestroy } from "@nestjs/common";
import type Redis from "ioredis";
import { EnvironmentService } from "../config/environment.service";
import { REDIS, createRedis } from "./redis";

@Global()
@Module({
  providers: [
    {
      provide: REDIS,
      inject: [EnvironmentService],
      useFactory: (environment: EnvironmentService) =>
        createRedis(environment.get("REDIS_URL")),
    },
  ],
  exports: [REDIS],
})
export class RedisModule implements OnModuleDestroy {
  constructor(@Inject(REDIS) private readonly redis: Redis) {}

  async onModuleDestroy(): Promise<void> {
    await this.redis.quit();
  }
}
