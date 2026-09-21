import { Module } from "@nestjs/common";
import { ConfigModule } from "./config/config.module";
import { DbModule } from "./db/db.module";
import { LoggerModule } from "./logger/logger.module";
import { RedisModule } from "./redis/redis.module";

@Module({ imports: [ConfigModule, LoggerModule, DbModule, RedisModule] })
export class CoreModule {}
