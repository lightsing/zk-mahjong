import { HardhatUserConfig, task } from 'hardhat/config'
import '@nomicfoundation/hardhat-toolbox'

require('dotenv').config()

const config: HardhatUserConfig = {
    solidity: {
        version: '0.8.23',
        settings: {
            optimizer: {
                enabled: true,
                runs: 200,
            },
        },
    },
    networks: {
        ganache: {
            url: 'http://127.0.0.1:8545',
            accounts: {
                mnemonic:
                    'test test test test test test test test test test test junk',
            },
        },
        sepolia: {
            url: 'https://rpc2.sepolia.org',
            accounts: {
                mnemonic: process.env.MNEMONIC,
            },
        },
    },
}

task('deploy', 'Deploy the contract')
    .addPositionalParam('name', 'The name of the contract')
    .setAction(async ({ name }, hre) => {
        console.log(`Deploy ${name} to ${hre.network.name}`)
        const factory = await hre.ethers.getContractFactory(
            `contracts/${name}.sol:PlonkVerifier`
        )
        const contract = await factory.deploy()
        await contract.waitForDeployment()
        console.log(`${name} deployed to:`, await contract.getAddress())
    })

export default config
