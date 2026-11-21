#!/bin/sh
# Creates a local self-signed code-signing certificate named
# "Miniwhispr Local Signing" in your login keychain. Run it once per Mac.
#
# Why: macOS ties Microphone and Accessibility permissions to the app's code
# signature. Unsigned or ad-hoc builds get a new signature on every rebuild, so
# macOS forgets the permissions. Signing every build with the same certificate
# keeps them. The certificate never leaves your Mac and is not an Apple
# Developer ID, so the app it signs is only for you.
set -eu

NAME="Miniwhispr Local Signing"
KEYCHAIN="$HOME/Library/Keychains/login.keychain-db"
# macOS ships LibreSSL here; its PKCS#12 output is what `security import` reads.
OPENSSL=/usr/bin/openssl

if security find-certificate -c "$NAME" "$KEYCHAIN" >/dev/null 2>&1; then
  echo "\"$NAME\" already exists in your login keychain. Nothing to do."
  exit 0
fi

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
pass=$(LC_ALL=C tr -dc 'A-Za-z0-9' </dev/urandom | head -c 32)

cat >"$tmp/cert.conf" <<EOF
[req]
distinguished_name = dn
x509_extensions = ext
prompt = no
[dn]
CN = $NAME
[ext]
basicConstraints = critical, CA:false
keyUsage = critical, digitalSignature
extendedKeyUsage = critical, codeSigning
EOF

"$OPENSSL" req -x509 -newkey rsa:2048 -nodes -days 3650 -sha256 \
  -config "$tmp/cert.conf" -keyout "$tmp/key.pem" -out "$tmp/cert.pem" 2>/dev/null
"$OPENSSL" pkcs12 -export -inkey "$tmp/key.pem" -in "$tmp/cert.pem" \
  -name "$NAME" -passout "pass:$pass" -out "$tmp/identity.p12"

# -T lets codesign use the private key.
security import "$tmp/identity.p12" -k "$KEYCHAIN" -P "$pass" -T /usr/bin/codesign

echo "macOS will now ask for your login password to trust the certificate for code signing."
security add-trusted-cert -r trustRoot -p codeSign -k "$KEYCHAIN" "$tmp/cert.pem"

echo "Done. Check it with: security find-identity -v -p codesigning"
