import { Global, Inject, Module, type OnModuleDestroy } from "@nestjs/common";
import { ThrottlerStorageRedisService } from "@nest-lab/throttler-storage-redis";
import { ThrottlerModule } from "@nestjs/throttler";
import type Redis from "ioredis";
import { EnvironmentService } from "../config/environment.service";
import { REDIS, createRedis } from "./redis";

@Global()
@Module({
  imports: [
    ThrottlerModule.forRootAsync({
      inject: [REDIS],
      useFactory: (redis: Redis) => ({
        throttlers: [{ name: "default", ttl: 10_000, limit: 100 }],
        storage: new ThrottlerStorageRedisService(redis),
      }),
    }),
  ],
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
    if (this.redis.status === "ready") {
      await this.redis.quit();
      return;
    }
    this.redis.disconnect();
  }
}
