import { describe, it, expect } from 'vitest';
import { getUser } from './register.test';
import * as auth from '$lib/stores/auth';
import * as crypto from '$lib/stores/cryptfns/rsa';

describe('Auth test', () => {
	it('API: Can login with credentials', async () => {
		const { user, password } = await getUser();
		const authenticated = await auth.login({
			email: user.email,
			password
		});
		expect(!!authenticated).toBeTruthy();
		const keypair = await crypto.get();
		expect(keypair).toBeTruthy();
	});
	it('API: Can not login with only email and password if the secure way of registering has been done (without encrypted secret on the server)', async () => {
		const { user, password } = await getUser(true);
		try {
			await auth.login({
				email: user.email,
				password
			});
		} catch (e) {
			expect((e as Error).message).toBe(
				'No encrypted secret key found on user from backend, not privateKey provided'
			);
		}
	});
	it('API: Can login with credentials and privateKey', async () => {
		const { user, password, privateKey } = await getUser(true);
		const authenticated = await auth.login({
			email: user.email,
			password,
			privateKey
		});
		expect(!!authenticated).toBeTruthy();
		const keypair = await crypto.get();
		expect(keypair).toBeTruthy();
	});
	it('API: Can login only with privateKey', async () => {
		const { user, privateKey } = await getUser(true);
		const authenticated = await auth.loginWithPrivateKey(privateKey);
		expect(!!authenticated).toBeTruthy();
		expect(authenticated.user.email).toBe(user.email);
		expect(authenticated.user.pubkey).toBe(user.pubkey);
		const keypair = await crypto.get();
		expect(keypair).toBeTruthy();
	});
});