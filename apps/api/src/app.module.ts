import { Module } from "@nestjs/common";
import { APP_FILTER, APP_GUARD, APP_PIPE } from "@nestjs/core";
import { BusinessErrorFilter } from "./common/filters/business-error.filter";
import { JwtAuthGuard } from "./common/guards/jwt-auth.guard";
import { OriginGuard } from "./common/guards/origin.guard";
import { validationPipe } from "./common/pipes/validation.pipe";
import { CoreModule } from "./core/core.module";
import { AuthModule } from "./features/auth/auth.module";
import { HealthModule } from "./features/health/health.module";

@Module({
  imports: [CoreModule, AuthModule, HealthModule],
  providers: [
    { provide: APP_FILTER, useClass: BusinessErrorFilter },
    { provide: APP_GUARD, useClass: OriginGuard },
    { provide: APP_GUARD, useClass: JwtAuthGuard },
    { provide: APP_PIPE, useFactory: validationPipe },
  ],
})
export class AppModule {}
