import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BinaryOptions } from "../target/types/binary_options";
import {
	createMint,
	getOrCreateAssociatedTokenAccount,
	mintTo,
	TOKEN_2022_PROGRAM_ID
} from "@solana/spl-token";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { assert } from "chai";
function sleep(ms) {
	return new Promise((resolve) => setTimeout(resolve, ms));
}
describe("binary_options", async () => {
	// Configure the client to use the local cluster.
	const provider = anchor.AnchorProvider.env();
	anchor.setProvider(anchor.AnchorProvider.env());
	let depositAccount, player1Account, player2Account, gameAccount;

	before(async () => {
		depositAccount = anchor.web3.Keypair.generate();
		player1Account = anchor.web3.Keypair.generate();
		player2Account = anchor.web3.Keypair.generate();
		gameAccount = anchor.web3.Keypair.generate();

		// Log funding information
		console.log("Funding accounts...");
		for (let account of [
			depositAccount,
			player1Account,
			player2Account,
			gameAccount
		]) {
			const airdropTx = await provider.connection.requestAirdrop(
				account.publicKey,
				2 * anchor.web3.LAMPORTS_PER_SOL
			); // Increased funding for transaction fees
			await provider.connection.confirmTransaction(
				airdropTx,
				"confirmed"
			);
			const balance = await provider.connection.getBalance(
				account.publicKey
			);
			console.log(
				`Account ${account.publicKey.toString()} funded with ${
					balance / anchor.web3.LAMPORTS_PER_SOL
				} SOL`
			);
		}
	});

	it("Is initialized!", async () => {
		const program = anchor.workspace
			.BinaryOptions as Program<BinaryOptions>;
		const usdc = anchor.web3.Keypair.generate();
		const mint = await createMint(
			program.provider.connection,
			depositAccount,
			depositAccount.publicKey,
			depositAccount.publicKey,
			9,
			usdc,
			{
				commitment: "confirmed"
			},
			TOKEN_2022_PROGRAM_ID
		);

		console.log(`Token created successfully: ${mint.toString()}`);
		const depositAccountAta = await getOrCreateAssociatedTokenAccount(
			program.provider.connection,
			depositAccount,
			mint,
			depositAccount.publicKey,
			true,
			"confirmed",
			{ commitment: "confirmed" },
			TOKEN_2022_PROGRAM_ID,
			ASSOCIATED_PROGRAM_ID
		);
		const responseFromMinTo = await mintTo(
			program.provider.connection,
			depositAccount,
			mint,
			depositAccountAta.address,
			depositAccount,
			200 * 10 ** 9,
			[],
			{ commitment: "confirmed" },
			TOKEN_2022_PROGRAM_ID
		);
		const usdcBalance =
			await program.provider.connection.getTokenAccountBalance(
				depositAccountAta.address
			);
		console.log(
			`Deposit account balance: ${usdcBalance.value.uiAmountString}`
		);

		assert.equal(usdcBalance.value.uiAmountString, "200");
		const tokenMint = new anchor.web3.PublicKey("token_mint_public_key");
		const feePercentage = 5; // Example fee percentage
		const tokenVault = new anchor.web3.PublicKey("token_vault_public_key");
		const tx = await program.methods
			.init(tokenMint, feePercentage, tokenVault)
			.rpc();
		console.log("Your transaction signature:", tx);
		console.log("Your transaction signature");
	});
});
