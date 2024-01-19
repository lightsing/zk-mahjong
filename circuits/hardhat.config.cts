import { HardhatUserConfig, task } from 'hardhat/config'
import '@nomicfoundation/hardhat-toolbox'
import { mkdirSync, existsSync } from 'node:fs'
import { execSync } from 'node:child_process'

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

task('compile-circuit', 'Compile the circuit')
  .addPositionalParam('name', 'The name of the circuit')
  .addParam('degree', 'The degree of the circuit')
  .setAction(async ({ name, degree }, hre) => {
    mkdirSync('cache', { recursive: true })
    mkdirSync(`build/${name}`, { recursive: true })
    if (!existsSync(`cache/${degree}.ptau`)) {
      execSync(`curl -L https://storage.googleapis.com/zkevm/ptau/powersOfTau28_hez_final_${degree}.ptau -o cache/${degree}.ptau`)
    }
    execSync(`circom circuits/${name}.circom --r1cs --sym --wasm --O2 --output build/${name}`)
    execSync(`snarkjs plonk setup build/${name}/${name}.r1cs cache/${degree}.ptau build/${name}/${name}.zkey`)
    execSync(`snarkjs zkey export verificationkey build/${name}/${name}.zkey build/${name}/verification_key.json`)
    execSync(`snarkjs zkey export solidityverifier build/${name}/${name}.zkey contracts/${name}.sol`)
  })


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
