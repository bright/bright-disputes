#![cfg_attr(not(feature = "std"), no_std, no_main)]

use baby_liminal_extension::VerificationKeyIdentifier;

mod contract;
mod dispute;
mod dispute_round;
mod error;
mod juror;
mod types;
mod vote;

const VOTE_VK_IDENTIFIER: VerificationKeyIdentifier =
    [b'v', b'o', b't', b'e', b'v', b'o', b't', b'e'];
const VERDICT_POSITIVE_VK_IDENTIFIER: VerificationKeyIdentifier =
    [b'v', b'e', b'r', b'd', b'i', b'c', b't', b'p'];
const VERDICT_NEGATIVE_VK_IDENTIFIER: VerificationKeyIdentifier =
    [b'v', b'e', b'r', b'd', b'i', b'c', b't', b'n'];
const VERDICT_NONE_VK_IDENTIFIER: VerificationKeyIdentifier =
    [b'v', b'e', b'r', b'd', b'i', b'c', b't', b'o'];
