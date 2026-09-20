import { Module } from "@nestjs/common";
import { TypeOrmModule } from "@nestjs/typeorm";
import { EnvironmentService } from "../config/environment.service";
import { postgresOptions } from "./data-source.options";

@Module({
  imports: [
    TypeOrmModule.forRootAsync({
      inject: [EnvironmentService],
      useFactory: (environment: EnvironmentService) => ({
        ...postgresOptions(environment.get("DATABASE_URL")),
        retryAttempts: 0,
      }),
    }),
  ],
})
export class DbModule {}
