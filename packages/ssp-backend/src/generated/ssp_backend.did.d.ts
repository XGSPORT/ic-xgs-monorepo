import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface Auth0JWK {
  'e' : string,
  'n' : string,
  'alg' : string,
  'kid' : string,
  'kty' : string,
  'use' : string,
  'x5c' : Array<string>,
  'x5t' : string,
}
export interface Auth0JWKS { 'keys' : Array<Auth0JWK> }
export interface Certificate {
  'user_principal' : Principal,
  'content' : CertificateContent,
  'created_at' : string,
  'managed_user_id' : [] | [string],
}
export interface CertificateContent {
  'issued_at' : string,
  'name' : string,
  'file_uri' : [] | [string],
  'issuer_club_name' : [] | [string],
  'issuer_full_name' : [] | [string],
  'notes' : [] | [string],
  'external_id' : [] | [string],
  'sport_category' : string,
}
export interface CertificatePreviewWithId { 'id' : string, 'name' : string }
export interface CertificateWithId {
  'id' : string,
  'certificate_cbor_hex' : string,
}
export interface Config { 'backend_principal' : [] | [Principal] }
export interface CreateCertificateContentRequest {
  'issued_at' : Timestamp,
  'name' : string,
  'file_uri' : [] | [string],
  'issuer_club_name' : [] | [string],
  'issuer_full_name' : [] | [string],
  'notes' : [] | [string],
  'external_id' : [] | [string],
  'sport_category' : string,
}
export interface CreateCertificateRequest {
  'content' : CreateCertificateContentRequest,
  'managed_user_db_id' : [] | [string],
  'user_db_id' : string,
}
export interface CreateCertificateResponse { 'id' : string }
export interface Delegation {
  'pubkey' : PublicKey,
  'targets' : [] | [Array<Principal>],
  'expiration' : Timestamp,
}
export interface GetCertificateResponse {
  'certificate' : CertificateWithId,
  'ic_certificate' : Uint8Array | number[],
  'ic_certificate_witness' : Uint8Array | number[],
}
export type GetDelegationResponse = { 'no_such_delegation' : null } |
  { 'signed_delegation' : SignedDelegation };
export interface GetUserCertificatesRequest {
  'user_principal' : [] | [Principal],
  'user_db_id' : [] | [string],
}
export interface GetUserCertificatesResponse {
  'certificates' : Array<CertificatePreviewWithId>,
}
export interface PrepareDelegationResponse {
  'user_key' : UserKey,
  'expiration' : Timestamp,
}
export type PublicKey = Uint8Array | number[];
export type Signature = Uint8Array | number[];
export interface SignedDelegation {
  'signature' : Signature,
  'delegation' : Delegation,
}
export type Timestamp = bigint;
export interface User {
  'sub' : string,
  'created_at' : string,
  'db_id' : string,
}
export type UserKey = PublicKey;
export interface _SERVICE {
  'create_certificate' : ActorMethod<
    [CreateCertificateRequest],
    CreateCertificateResponse
  >,
  'get_certificate' : ActorMethod<[string], GetCertificateResponse>,
  'get_config' : ActorMethod<[], Config>,
  'get_delegation' : ActorMethod<[string, Timestamp], GetDelegationResponse>,
  'get_jwks' : ActorMethod<[], [] | [Auth0JWKS]>,
  'get_my_user' : ActorMethod<[], User>,
  'get_user_certificates' : ActorMethod<
    [GetUserCertificatesRequest],
    GetUserCertificatesResponse
  >,
  'prepare_delegation' : ActorMethod<[string], PrepareDelegationResponse>,
  'set_backend_principal' : ActorMethod<[Principal], undefined>,
  'set_jwks' : ActorMethod<[Auth0JWKS], undefined>,
  'sync_jwks' : ActorMethod<[], undefined>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
