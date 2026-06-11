export interface VaultStatusDto {
  locked: boolean;
  key_count: number;
  auto_lock_remaining: number;
  providers_with_keys: { id: string }[];
}

export interface VaultUnlockDto {
  success: boolean;
}

export interface VaultMigrateDto {
  migrated: string[];
}

export interface VaultAutoLockDto {
  minutes: number;
}
