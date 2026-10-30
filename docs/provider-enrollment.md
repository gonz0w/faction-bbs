# Manual same-home provider enrollment

This software-preview workflow connects a member's native attachment client to
its own home's loopback provider through a dedicated SSH local forward. The owner
must separately authorize any real account and service activation. It uses the
existing [offline preparation](provider-preparation.md),
[provider operator](provider-operator.md), and
[private attachment](private-attachments.md) interfaces; there is no automated
issuer or enrollment command. Both participants need independently trusted
communications and an already usable member vault and home mail setup.

There are two separate trust lanes. The SSH key permits only reachability to the
specified home-loopback port. The provider authenticates the client TLS key and
maps the exact client certificate's SHA-256 DER digest to an approved account,
device and full public identity. SSH access alone creates no provider principal.
Ordinary encrypted mail continues over its separate XMPP route.

## 1. Member: create a transport key and request

On the member's own computer, in a private directory, generate a client TLS key
and signed PKCS#10 certificate request. This example uses the same P-256 and
SHA-256 choices as the software fixture; select fresh paths and protect the key:

```sh
umask 077
cat > request.cnf <<'CONFIG'
[req]
distinguished_name=dn
[dn]
CONFIG
openssl req -config request.cnf -new -newkey ec -pkeyopt ec_paramgen_curve:P-256 \
  -nodes -sha256 -subj '/CN=member-generated-transport' \
  -keyout device-key.pem -out device.csr
```

This private request configuration avoids relying on or modifying a system OpenSSL
configuration. It does not replace CA trust or certificate verification.
The unencrypted key stays on the member device. Send only the public CSR through
the independently trusted operator channel. The request signature proves control
of the TLS key at request time; it does not prove control of the BBS account or
OMEMO private key. The first successful mutual TLS operation proves possession of
the TLS key as well, but does not prove the OMEMO identity.

## 2. Operator: verify, approve and issue

Create the same minimal request configuration in a private operator directory,
then verify the public CSR's signature:

```sh
umask 077
cat > request.cnf <<'CONFIG'
[req]
distinguished_name=dn
[dn]
CONFIG
openssl req -config request.cnf -in device.csr -verify -noout
```

Independently verify the member's account control, full public OMEMO identity and
device number, and the intended client certificate digest through your approval
process. A subject name, CSR, public card, SSH key or password alone is not that
approval. An incorrectly approved identity can be asserted by a TLS-key holder;
TLS cannot repair the operator's verification. A password reset neither enrolls
a device nor recovers its keys.

An independently supplied offline CA issues the public client certificate from
the verified CSR. Use a unique serial and the CA's established issuance and
validity procedures. The test profile uses these extensions in `client.ext`:

```text
basicConstraints=critical,CA:FALSE
extendedKeyUsage=clientAuth
keyUsage=critical,digitalSignature
```

For example, with a suitable independently maintained CA and unique serial:

```sh
openssl x509 -req -in device.csr -CA ca.pem -CAkey ca-key.pem \
  -set_serial UNIQUE_SERIAL -days VALID_DAYS -sha256 \
  -extfile client.ext -out device.pem
```

Replace `UNIQUE_SERIAL` and `VALID_DAYS` with actual values. Keep the CA key
under operator control; return only the public client certificate and trusted CA
certificate. Independently communicate the home TLS name and server leaf pin.
Check that the certificate is the one approved for the member. The member's CSR,
client certificate and public card are the enrollment artifacts exchanged across
the member/operator boundary; never send the member's private key or profile.

## 3. Member: prepare a profile and card

Place the CA certificate, client certificate and matching key under the private
profile parent. Follow the ownership, permission, bounded-file and template rules
in [offline preparation](provider-preparation.md). Set `endpoint` to the local
end of the SSH forward, for example `127.0.0.1:12345`. Keep `home` and `name` as
the verified remote home and TLS name, and keep the independently checked server
leaf pin and CA: forwarding does not change TLS identity checks.

With the existing vault, private template and fresh output paths, run on the
member's computer:

```sh
faction-client --device-provider-prepare VAULT_DIR PRIVATE_TEMPLATE PRIVATE_PROFILE_OUTPUT
faction-client --device-provider-card VAULT_DIR PRIVATE_PROFILE_OUTPUT PRIVATE_CARD_OUTPUT
```

Unlock locally when prompted. The card contains the account, device, full public
identity and client certificate digest; it is self-asserted and does not approve
itself. Send the card and public certificate to the trusted operator. Retain the
private profile and key locally. Each output requires a new path and publication
is not a transaction; consult the preparation guide before retrying a failure.

## 4. Operator: check and activate the complete policy

Prepare a complete independently approved proposal, including every retained
binding, flag, validity interval, generation and reset epoch. Compare the card,
certificate digest and identity with independently verified values. Keep a
protected backup of the full proposal and compare changes with retained state:
`prepare` neither exports nor merges the current registry. The operator guide
specifies paths, file permissions, proposal fields and bounds.

```sh
faction-provider prepare /private/operator/config.toml /private/operator/card.toml /private/operator/client.pem /private/operator/approved.toml /private/operator/candidate.toml
```

This offline comparison produces a candidate; it does not approve or apply it.
Review the entire candidate. For a new empty policy root, explicitly initialize
it; for an established root, stop the serving owner before applying a complete
higher-generation proposal. Use one applicable action, then inspect and serve:

```sh
faction-provider init /private/operator/config.toml /private/operator/candidate.toml
# Or, after stopping an established owner:
faction-provider apply /private/operator/config.toml /private/operator/candidate.toml
faction-provider inspect /private/operator/config.toml
faction-provider serve /private/operator/config.toml
```

`apply` replaces the whole registry. The current floors and certificate history
can still reject a candidate; do not delete or reinitialize established state to
bypass a refusal. There is no online policy reload. The operator guide describes
uncertain publication, recovery and the foreground service's stop behavior.

## 5. Member: open the dedicated forward and use attachments

The sysop separately authorizes a dedicated forwarding-only SSH account and key.
Restrict it to local TCP forwarding and an exact `PermitOpen` for the provider's
loopback host and port; deny shell and subsystem sessions, agent and X11
forwarding, StreamLocal and tunnel forwarding. `MaxSessions 0` blocks session
channels but is not a forwarding-channel or connection limit. Keep ordinary BBS
and SFTP accounts' `DisableForwarding` policy intact. See the OpenSSH
[server configuration reference](https://man.openbsd.org/sshd_config).

After independently verifying the SSH host key, replace these placeholders with
the authorized key, account, host and actual ports. Keep this command running:

```sh
ssh -N -T -i /private/member/forward-key -p SSH_PORT \
  -o IdentitiesOnly=yes -o ExitOnForwardFailure=yes \
  -o ForwardAgent=no -o ForwardX11=no \
  -L 127.0.0.1:LOCAL_PORT:127.0.0.1:PROVIDER_PORT forward-user@home-host
```

The local port must match the member profile; the destination must match the
sysop's exact permitted home-loopback endpoint. `ExitOnForwardFailure` confirms
forward setup, not provider readiness or a successful TLS handshake. See the
[OpenSSH client reference](https://man.openbsd.org/ssh). Use the existing native
client with the prepared profile and the attachment desk to choose a file, queue,
upload, fetch, verify and explicitly save as described in
[private attachments](private-attachments.md). A selected provider failure stops
the attachment action; it does not fall back to another route.

## Replacement and reset

For rotation, independently approve a new certificate and prepare the complete
replacement proposal with a higher generation, preserving all other intended
bindings and retained history. Stop the owner, apply, inspect and restart it; a
certificate digest cannot be remapped to a different principal. A policy reset
clears all active bindings and requires fresh approval; it does not recover an
OMEMO key or old encrypted history. A lost identity requires renewed peer trust.
Follow the [operator recovery instructions](provider-operator.md#commands-and-recovery)
for reset floors and uncertain state.

This same-home preview does not establish cross-home delegation, independent-client
interoperability or a deployed provider. It does not claim physical-hardware or
250-member validation. Source research and software evidence do not themselves
authorize a real host change.

The [software qualification](../dev-docs/evidence/provider-enrollment/README.md)
records the actual classic/Unicode host and NetBSD controls, executed identities,
same-UID SSH fixture limit and retained privacy-scan limits.
