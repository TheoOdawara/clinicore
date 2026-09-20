import { Module } from "@nestjs/common";
import { ConfigModule } from "./config/config.module";
import { DbModule } from "./db/db.module";
import { LoggerModule } from "./logger/logger.module";

@Module({ imports: [ConfigModule, LoggerModule, DbModule] })
export class CoreModule {}
