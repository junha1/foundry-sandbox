// Copyright 2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

pub mod intra;
pub mod multiplex;
pub mod stream_socket;
pub mod udp;

use once_cell::sync::OnceCell;
use parking_lot::Mutex;
pub use remote_trait_object::transport::{Terminate, TransportError, TransportRecv, TransportSend};
use sha2::Digest;

pub trait Ipc: Sized + TransportSend + TransportRecv {
    /// Generate two configurations
    /// which will be feeded to Ipc::new(),
    /// for both two ends in two different processes, to initialize each IPC end.
    /// Note that both sides' new() must be called concurrently; they will be completed only if
    /// both run at the same time.
    fn arguments_for_both_ends() -> (Vec<u8>, Vec<u8>);

    type SendHalf: TransportSend;
    type RecvHalf: TransportRecv;

    /// Constructs itself with an opaque data that would have been transported by some IPC
    fn new(data: Vec<u8>) -> Self;

    /// split itself into Send-only and Recv-only. This is helpful for a threading
    /// When you design both halves, you might consider who's in charge of cleaning up things.
    /// Common implementation is making both to have Arc<SomethingDroppable>.
    fn split(self) -> (Self::SendHalf, Self::RecvHalf);
}

/// Most of IPC depends on a system-wide name, which looks quite vulnerable for
/// possible attack. Rather, generating a random name would be more secure.
pub fn generate_random_name() -> String {
    static MONOTONIC: OnceCell<Mutex<u64>> = OnceCell::new();
    let mut mono = MONOTONIC.get_or_init(|| Mutex::new(0)).lock();
    *mono += 1;
    let mono = *mono;
    let pid = std::process::id();
    let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap();
    let mut hasher = sha2::Sha256::new();
    hasher.update(format!("{:?}{}{}", time, pid, mono));
    let hash = hasher.finalize();
    hex::encode(hash.as_slice())[0..32].to_string()
}
