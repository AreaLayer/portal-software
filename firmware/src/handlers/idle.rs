// Portal Hardware Wallet firmware and supporting software libraries
//
// Copyright (C) 2024 Alekos Filini
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use alloc::rc::Rc;

use futures::prelude::*;

use gui::{InitialPage, SummaryPage};
use model::{DeviceInfo, Reply};

use super::*;
use crate::Error;

pub async fn handle_idle(
    wallet: &mut Rc<PortalWallet>,
    mut events: impl Stream<Item = Event> + Unpin,
    peripherals: &mut HandlerPeripherals,
) -> Result<CurrentState, Error> {
    log::info!("handle_idle");

    let page = InitialPage::new("Portal ready", "");
    page.init_display(&mut peripherals.display)?;
    page.draw_to(&mut peripherals.display)?;
    peripherals.display.flush()?;

    let events = only_requests(&mut events);
    pin_mut!(events);

    loop {
        match events.next().await {
            Some(model::Request::GetInfo) => {
                peripherals
                    .nfc
                    .send(Reply::Info(DeviceInfo::new_unlocked_initialized(
                        wallet.network(),
                        wallet.xprv.fingerprint(wallet.secp_ctx()).to_bytes(),
                        env!("CARGO_PKG_VERSION"),
                    )))
                    .await
                    .unwrap();
                peripherals.nfc_finished.recv().await.unwrap();
                continue;
            }
            Some(model::Request::DisplayAddress(index)) => {
                break Ok(CurrentState::DisplayAddress {
                    index,
                    wallet: Rc::clone(wallet),
                    resumable: checkpoint::Resumable::fresh(),
                    is_fast_boot: false,
                });
            }
            Some(model::Request::BeginSignPsbt) => {
                break Ok(CurrentState::WaitingForPsbt {
                    wallet: Rc::clone(wallet),
                });
            }
            Some(model::Request::PublicDescriptor) => {
                break Ok(CurrentState::PublicDescriptor {
                    wallet: Rc::clone(wallet),
                    resumable: checkpoint::Resumable::fresh(),
                    is_fast_boot: false,
                });
            }
            Some(model::Request::GetXpub(derivation_path)) => {
                break Ok(CurrentState::GetXpub {
                    wallet: Rc::clone(wallet),
                    derivation_path: derivation_path.into(),
                    resumable: checkpoint::Resumable::fresh(),
                    is_fast_boot: false,
                    encryption_key: checkpoint::Checkpoint::gen_key(&mut peripherals.rng),
                });
            }
            Some(model::Request::SetDescriptor {
                variant,
                script_type,
                bsms,
            }) => {
                break Ok(CurrentState::SetDescriptor {
                    wallet: Rc::clone(wallet),
                    variant,
                    script_type,
                    bsms,
                    resumable: checkpoint::Resumable::fresh(),
                    is_fast_boot: false,
                    encryption_key: checkpoint::Checkpoint::gen_key(&mut peripherals.rng),
                });
            }
            Some(model::Request::BeginFwUpdate(header)) => {
                break Ok(CurrentState::UpdatingFw {
                    header,
                    fast_boot: None,
                });
            }

            Some(model::Request::WipeDevice) => break Ok(CurrentState::WipeDevice),

            Some(model::Request::ShowMnemonic) => {
                break Ok(CurrentState::ShowMnemonic {
                    wallet: Rc::clone(wallet),
                    resumable: checkpoint::Resumable::fresh(),
                    is_fast_boot: false,
                });
            }

            Some(_) => {
                peripherals
                    .nfc
                    .send(model::Reply::UnexpectedMessage)
                    .await
                    .unwrap();
                peripherals.nfc_finished.recv().await.unwrap();
                continue;
            }
            _ => unreachable!(),
        }
    }
}

pub async fn wipe_device(
    mut events: impl Stream<Item = Event> + Unpin,
    peripherals: &mut HandlerPeripherals,
) -> Result<CurrentState, Error> {
    let mut page = SummaryPage::new_with_threshold("Wipe device?", "HOLD BTN TO WIPE", 70);
    page.init_display(&mut peripherals.display)?;
    page.draw_to(&mut peripherals.display)?;
    peripherals.display.flush()?;

    peripherals.tsc_enabled.enable();
    manage_confirmation_loop(&mut events, peripherals, &mut page).await?;
    peripherals.tsc_enabled.disable();

    crate::hw::write_flash(&mut peripherals.flash, crate::config::CONFIG_PAGE, &[])?;

    peripherals.nfc.send(model::Reply::Ok).await.unwrap();
    peripherals.nfc_finished.recv().await.unwrap();

    cortex_m::peripheral::SCB::sys_reset();
}
