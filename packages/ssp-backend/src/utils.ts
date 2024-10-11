import {
  Cbor,
  Certificate,
  HttpAgent,
  compare,
  lookupResultToBuffer,
  lookup_path,
  reconstruct,
  type HashTree,
  fromHex,
  NodeType,
} from "@dfinity/agent";
import type { CertificateWithId } from "./generated/ssp_backend.did";
import { Principal } from "@dfinity/principal";

export type DecodedCertificate = {
  content: {
    name: string;
    issued_at: string;
    sport_category: string;
    notes: string | null;
    file_uri: string | null;
    external_id: string | null;
    issuer_full_name: string | null;
    issuer_club_name: string | null;
  };
  created_at: string;
  managed_user_id: string | null;
  user_principal: Principal;
};

export const optionalToNullable = <T>(
  value: [] | [T] | null | undefined,
): T | null => {
  if (!value) {
    return null;
  }
  return value.length === 1 ? value[0] : null;
};

export const nullableToOptional = <T>(
  value: T | null | undefined,
): [T] | [] => {
  return value ? [value] : [];
};

export const parseUint8Array = (
  other: Uint8Array | number[] | Record<string, number>,
): Uint8Array => {
  if (Object.getPrototypeOf(other) === Uint8Array.prototype) {
    return other as Uint8Array;
  }
  if (Array.isArray(other)) {
    return new Uint8Array(other);
  }
  return new Uint8Array(Object.values(other));
};

const parsePrincipal = (principal: string | Principal | object): Principal => {
  if (principal instanceof Principal) {
    return principal;
  }
  if (Object.getPrototypeOf(principal) === Uint8Array.prototype) {
    return Principal.fromUint8Array(principal as Uint8Array);
  }
  return Principal.from(JSON.stringify(principal));
};

export const decodeCertificate = (
  certCbor: string | ArrayBuffer,
): DecodedCertificate => {
  const cbor = typeof certCbor === "string" ? fromHex(certCbor) : certCbor;
  const decoded = Cbor.decode<DecodedCertificate>(cbor);
  return {
    ...decoded,
    user_principal: parsePrincipal(decoded.user_principal),
  };
};

const uuidToBytes = (uuid: string): Uint8Array => {
  // Remove hyphens and convert to lowercase
  const hex = uuid.replace(/-/g, "").toLowerCase();

  // Convert hex string to byte array
  const bytes = new Uint8Array(16);
  for (let i = 0; i < 16; i++) {
    bytes[i] = parseInt(hex.slice(i * 2, i * 2 + 2), 16);
  }

  return bytes;
};

const areBuffersEqual = (buf1: ArrayBuffer, buf2: ArrayBuffer): boolean => {
  return compare(buf1, buf2) === 0;
};

const MAX_CERTIFICATE_AGE_IN_MINUTES = 5;
const SSP_CERTIFICATES_TREE_LABEL = "ssp_certificates"; // same as in the canister

export const verifyCertificateIntegrity = async (
  data: CertificateWithId,
  icCertificate: ArrayBuffer,
  icCertificateWitness: ArrayBuffer,
  canisterId: string,
  icHost: string,
  isMainnet: boolean,
): Promise<boolean> => {
  const cid = Principal.fromText(canisterId);

  const agent = await HttpAgent.create({
    host: icHost,
    shouldFetchRootKey: !isMainnet,
  });

  let cert;
  try {
    cert = await Certificate.create({
      certificate: icCertificate,
      canisterId: cid,
      rootKey: agent.rootKey!,
      maxAgeInMinutes: MAX_CERTIFICATE_AGE_IN_MINUTES,
    });
  } catch (error) {
    console.error("[certification] Error creating certificate:", error);
    return false;
  }

  const hashTree = Cbor.decode<HashTree>(icCertificateWitness);
  const reconstructed = await reconstruct(hashTree);
  const witnessLookupResult = cert.lookup([
    "canister",
    cid.toUint8Array(),
    "certified_data",
  ]);
  const witness = lookupResultToBuffer(witnessLookupResult);

  if (!witness) {
    console.error(
      "[certification] Could not find certified data for this canister in the certificate.",
    );
    return false;
  }

  // First validate that the Tree is as good as the certification.
  if (!areBuffersEqual(witness, reconstructed)) {
    console.error("[certification] Witness != Tree passed in ic-certification");
    return false;
  }

  const certCbor = fromHex(data.certificate_cbor_hex);
  const certificateData = decodeCertificate(certCbor);

  // Next, calculate the SHA of the content.
  const sha = await reconstruct([NodeType.Leaf, certCbor]);
  const path = [
    SSP_CERTIFICATES_TREE_LABEL,
    certificateData.user_principal.toUint8Array(),
    uuidToBytes(data.id),
  ];
  let treeShaLookupResult = lookup_path(path, hashTree);
  const treeSha = lookupResultToBuffer(treeShaLookupResult);

  if (!treeSha) {
    // The tree returned in the certification header is wrong. Return false.
    // We don't throw here, just invalidate the request.
    console.error(
      "[certification] Invalid Tree in the header. Does not contain path",
      path,
    );
    return false;
  }

  const verified = areBuffersEqual(sha, treeSha as ArrayBuffer);
  if (!verified) {
    console.error("[certification] SHA does not match tree SHA");
  }
  return verified;
};
