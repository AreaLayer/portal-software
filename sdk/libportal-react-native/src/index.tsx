import { NativeModules, Platform } from 'react-native';

const LINKING_ERROR =
  `The package 'libportal-react-native' doesn't seem to be linked. Make sure: \n\n` +
  Platform.select({ ios: "- You have run 'pod install'\n", default: '' }) +
  '- You rebuilt the app after installing the package\n' +
  '- You are not using Expo Go\n';

const LibportalReactNative = NativeModules.LibportalReactNative
  ? NativeModules.LibportalReactNative
  : new Proxy(
      {},
      {
        get() {
          throw new Error(LINKING_ERROR);
        },
      }
    );

export class PortalSdk {
  constructor(useFastOps: boolean) {
    LibportalReactNative.constructor(useFastOps)
      .catch((e: any) => console.warn(e))
  }

  destroy() {
    LibportalReactNative.destructor()
  }

  poll(): Promise<NfcOut> {
    return LibportalReactNative.poll()
  }

  newTag(): Promise<void> {
    return LibportalReactNative.newTag()
  }

  incomingData(msgIndex: number, data: number[]): Promise<void> {
    return LibportalReactNative.incomingData(msgIndex, data)
  }

  getStatus(): Promise<CardStatus> {
    return LibportalReactNative.getStatus()
      .then((status: any) => {
        if (status.network) {
          status.network = status.network as Network;
        }

        return status;
      });
  }

  generateMnemonic(words: MnemonicWords, network: Network, pair_code?: string): Promise<void> {
    return LibportalReactNative.generateMnemonic(words.toString(), network.toString(), pair_code);
  }

  restoreMnemonic(mnemonic: string, network: Network, pair_code?: string): Promise<void> {
    return LibportalReactNative.restoreMnemonic(mnemonic, network.toString(), pair_code);
  }

  unlock(pair_code: string): Promise<void> {
    return LibportalReactNative.unlock(pair_code);
  }

  resume(): Promise<void> {
    return LibportalReactNative.resume();
  }

  showMnemonic(): Promise<void> {
    return LibportalReactNative.showMnemonic();
  }

  displayAddress(index: number): Promise<string> {
    return LibportalReactNative.displayAddress(index);
  }

  signPsbt(psbt: string): Promise<string> {
    return LibportalReactNative.signPsbt(psbt);
  }

  publicDescriptors(): Promise<Descriptors> {
    return LibportalReactNative.publicDescriptors();
  }

  getXpub(derivationPath: string): Promise<GetXpubData> {
    return LibportalReactNative.getXpub(derivationPath);
  }

  setDescriptor(descriptor: string, bsms?: SetDescriptorBsmsData): Promise<void> {
    let bsmsSerialized = null;
    if (bsms) {
      bsmsSerialized = JSON.stringify(bsms);
    }

    return LibportalReactNative.setDescriptor(descriptor, bsmsSerialized);
  }

  updateFirmware(bytes: number[]): Promise<void> {
    return LibportalReactNative.updateFirmware(bytes);
  }

  debugWipeDevice(): Promise<void> {
    return LibportalReactNative.debugWipeDevice();
  }
}

export enum Network {
  Bitcoin = 'bitcoin',
  Testnet = 'testnet',
  Regtest = 'regtest',
  Signet = 'signet',
}

export enum MnemonicWords {
  Words12,
  Words24,
}

export interface Descriptors {
  readonly external: string,
  readonly internal?: string,
}

export interface NfcOut {
  readonly msgIndex: number,
  readonly data: number[],
}

export interface CardStatus {
  readonly initialized: boolean,
  readonly unverified?: boolean,
  readonly unlocked: boolean,
  readonly network?: Network,
}

export interface GetXpubData {
  readonly xpub: string,
  readonly bsms: string,
}

export interface GetXpubBsms {
  readonly version: string,
  readonly token: string,
  readonly keyName: string,
  readonly signature: string,
}

export interface SetDescriptorBsmsData {
  readonly version: string,
  readonly pathRestrictions: string,
  readonly firstAddress: string,
}
