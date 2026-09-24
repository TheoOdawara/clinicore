import {
  Column,
  CreateDateColumn,
  Entity,
  Index,
  JoinColumn,
  ManyToOne,
  PrimaryGeneratedColumn,
  Unique,
  UpdateDateColumn,
} from "typeorm";
import { SessionClient } from "../enums/session-client.enum";
import { User } from "./user.entity";

@Entity("session")
@Unique(["refreshTokenHash"])
export class Session {
  @PrimaryGeneratedColumn("uuid")
  id!: string;

  @Index()
  @Column({ type: "uuid" })
  userId!: string;

  @Column({ type: "varchar", length: 64 })
  refreshTokenHash!: string;

  @Index()
  @Column({ type: "timestamptz" })
  expiresAt!: Date;

  @Column({ type: "varchar", length: 45, nullable: true })
  ipAddress!: string | null;

  @Column({ type: "varchar", length: 512, nullable: true })
  userAgent!: string | null;

  @Column({ type: "enum", enum: SessionClient, default: SessionClient.Web })
  client!: SessionClient;

  @CreateDateColumn({ type: "timestamptz" })
  createdAt!: Date;

  @UpdateDateColumn({ type: "timestamptz" })
  updatedAt!: Date;

  @ManyToOne(() => User, (user) => user.sessions, { onDelete: "CASCADE" })
  @JoinColumn({ name: "userId" })
  user!: User;
}
