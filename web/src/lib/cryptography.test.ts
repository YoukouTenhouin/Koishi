import { restricted_hash } from './cryptography'

test('Restricted hash test 1', async () => {
    expect(await restricted_hash("019627e8561f77b1b97022f0cdefeaf6", "114514"))
        .toBe("b49a98edc7716bfab1e3208694167b737379abf382d0ede02a023fff7f5008bc")
    expect(await restricted_hash("019627e86239727195b8b4a729d1ae31", "1919810"))
        .toBe("403f98674ad128812c24299d424e32d8e5cebb8702ec2635baa45fa939ae81b0")
})
