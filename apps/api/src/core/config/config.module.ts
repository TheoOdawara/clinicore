import { Global, Module } from "@nestjs/common";
import { ConfigModule as NestConfigModule } from "@nestjs/config";
import { EnvironmentService } from "./environment.service";
import { validateEnv } from "./env.validation";

@Global()
@Module({
  imports: [NestConfigModule.forRoot({ validate: validateEnv })],
  providers: [EnvironmentService],
  exports: [EnvironmentService],
})
export class ConfigModule {}
