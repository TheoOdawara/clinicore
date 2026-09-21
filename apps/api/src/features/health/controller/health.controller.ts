import { Controller, Get } from "@nestjs/common";
import { ApiTags } from "@nestjs/swagger";
import { SkipThrottle } from "@nestjs/throttler";
import { Public } from "../../../common/decorators/public.decorator";
import {
  HealthCheck,
  HealthCheckService,
  type HealthCheckResult,
} from "@nestjs/terminus";

@Public()
@SkipThrottle()
@ApiTags("Health")
@Controller("health")
export class HealthController {
  constructor(private readonly health: HealthCheckService) {}

  @Get()
  @HealthCheck()
  check(): Promise<HealthCheckResult> {
    return this.health.check([]);
  }
}
