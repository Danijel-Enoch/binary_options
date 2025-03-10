import {
	Account,
	Pubkey,
	Result,
	i64,
	u8,
	Signer,
	u64,
	TokenAccount,
	UncheckedAccount,
	Boolean,
	Str,
	f64,
	u128,
	AssociatedTokenAccount,
	Mint,
	TokenProgram,
	Seeds
} from "@solanaturbine/poseidon";

// creating a class VoteProgram is similar to creating a creating a mod in anchor with all the instructions inside
export default class BinaryOptions {
	// define the progam id as a static constant like bellow
	static PROGRAM_ID = new Pubkey(
		"HC2oqz2p6DEWfrahenqdq2moUcga9c9biqRBcdK3XKU1"
	);

	// we can pass in standard Accounts(Signer, TokenAccount, Mint, UncheckedAccount and so on), Custom Accounts(state in this case) and IX arguements(hash in this case) as parameters.
	initialize(
		state: BinaryOptionsState,
		user: Signer,
		xyzVault: TokenAccount,
		auth: UncheckedAccount,
		xyzMint: Mint
	): Result {
		// PDAs can be derived like <custom_Acc>.derive([...])
		// where inside array we can pass string, Uint8Array, pubkey
		// we can also derive PDAs which are token account, associated token account which will be covered in vault and escrow
		auth.derive(["auth"]);
		state.derive(["binary_options", auth.key]).init(user); // we can initialise PDA just by chaining a init method to the derive method
		xyzVault.derive(["vault", state.key], xyzMint, auth.key);
		// defining properties(vote) of custom_Acc(state)
		state.total_xyz_balance = new u64(0);
		state.fee_percentage = new u64(0);
		state.admin = user.key;
		state.xyz_mint = xyzMint.key;
		state.authBump = auth.getBump();
		state.prediction_counter = new u64(1);
	}
	create_prediction(
		user: Signer,
		auth: UncheckedAccount,
		state: BinaryOptionsState,
		xyz_mint: Mint,
		prediction_state: PredictionState,
		makerAta: AssociatedTokenAccount,
		xyz_vault: TokenAccount,
		amount: u64,
		token_mint: Pubkey,
		start_timestamp: u64,
		expiry_timestamp: u64,
		start_price: u64,
		end_price: u64,
		prediction_type: Str<6>
	): Result {
		auth.derive(["auth"]);
		state.derive(["binary_options", auth.key]);
		xyz_vault.derive(["vault", state.key], xyz_mint, auth.key);
		// create prediction by sending funds into smart contract
		makerAta.derive(xyz_mint, user.key);
		prediction_state
			.derive(["prediction", state.prediction_counter])
			.init(user);
		TokenProgram.transfer(
			makerAta, // from
			xyz_vault, // to
			user, // authority
			amount // amount to transferred
		);

		prediction_state.amount = amount;
		prediction_state.token_mint = token_mint;
		prediction_state.start_timestamp = start_timestamp;
		prediction_state.expiry_timestamp = expiry_timestamp;
		prediction_state.start_price = start_price;
		prediction_state.end_price = end_price;
		prediction_state.prediction_type = prediction_type;
		prediction_state.trader = makerAta.key;
		prediction_state.is_settled = false;
		prediction_state.is_winning = false;
		//
		state.prediction_counter.add(1);
	}
	settle_prediction(
		auth: UncheckedAccount,
		state: BinaryOptionsState,
		prediction_state: PredictionState,
		xyzVault: TokenAccount,
		takerReceiveAta: TokenAccount,
		user: Signer,
		xyz_mint: Mint,
		is_winning: Boolean,
		//id: u64,
		taker: Pubkey
	): Result {
		auth.derive(["auth"]);
		state.derive(["binary_options", auth.key]);
		prediction_state.derive(["prediction"]);

		prediction_state.is_settled = true;
		//prediction_state.is_winning = is_winning;
		xyzVault.derive(["vault", state.key], xyz_mint, auth.key);
		let seeds: Seeds = ["auth", state.authBump.toBytes()];
		// TokenProgram.transfer(
		// 	xyzVault, // from
		// 	takerReceiveAta, // to
		// 	auth, // authority
		// 	prediction_state.amount, // amount to be sent
		// 	seeds // seeds will be at the last arguments if needed
		// );
		// only admin can settle predicti	on
		// check user has been settled
		// settle user by sending tokens to their account
		// send to user original input amount - 2 percent fees
		// send to user 15 %
	}
}

// define custom accounts by creating an interface which extends class Account

export interface BinaryOptionsState extends Account {
	admin: Pubkey;
	prediction_counter: u64;
	xyz_mint: Pubkey;
	xyz_vault: Pubkey;
	total_xyz_balance: u64;
	fee_percentage: u64;
	authBump: u8;
	vaultBump: u8;
}

export interface PredictionState extends Account {
	user: Pubkey;
	amount: u64;
	trader: Pubkey;
	token_mint: Pubkey;
	start_timestamp: u64;
	expiry_timestamp: u64;
	start_price: u64;
	end_price: u64;
	prediction_type: Str<6>;
	is_settled: Boolean;
	is_winning: Boolean;
}
