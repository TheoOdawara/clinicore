import { MigrationInterface, QueryRunner } from "typeorm";

export class AddSessionClient1790235578055 implements MigrationInterface {
  name = "AddSessionClient1790235578055";

  public async up(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(
      `CREATE TYPE "public"."session_client_enum" AS ENUM('web', 'mobile')`,
    );
    await queryRunner.query(
      `ALTER TABLE "session" ADD "client" "public"."session_client_enum" NOT NULL DEFAULT 'web'`,
    );
  }

  public async down(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(`ALTER TABLE "session" DROP COLUMN "client"`);
    await queryRunner.query(`DROP TYPE "public"."session_client_enum"`);
  }
}
