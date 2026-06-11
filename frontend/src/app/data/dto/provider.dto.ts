export interface ProviderListDto {
  providers: ProviderItemDto[];
}

export interface ProviderItemDto {
  id: string;
  name: string;
  name_zh?: string;
  api_type: string;
  base_url: string;
  models: string[];
}

export interface ProviderConfigStatusDto {
  primary: { provider_id: string; model: string } | null;
  secondary: { provider_id: string; model: string } | null;
  configured_providers: { id: string; has_key: boolean; base_url?: string }[];
}

export interface ProviderTestDto {
  provider_id: string;
  api_key?: string;
  base_url?: string;
}

export interface ProviderTestResultDto {
  valid: boolean;
  error?: string;
}

export interface ProviderConfigureDto {
  provider_id: string;
  api_key?: string;
  base_url?: string;
  is_primary?: boolean;
  is_secondary?: boolean;
  model?: string;
}

export interface TestAllKeysResultDto {
  results: Record<string, { valid: boolean }>;
}
