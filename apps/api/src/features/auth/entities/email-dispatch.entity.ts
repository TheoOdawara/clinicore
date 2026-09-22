import {
  Column,
  CreateDateColumn,
  Entity,
  Index,
  PrimaryGeneratedColumn,
} from "typeorm";
import { EmailDispatchKind } from "../enums/email-dispatch-kind.enum";

@Entity("emailDispatch")
@Index(["email", "kind", "createdAt"])
export class EmailDispatch {
  @PrimaryGeneratedColumn("uuid")
  id!: string;

  @Column({ type: "varchar", length: 320 })
  email!: string;

  @Column({ type: "enum", enum: EmailDispatchKind })
  kind!: EmailDispatchKind;

  @Index()
  @CreateDateColumn({ type: "timestamptz" })
  createdAt!: Date;
}
