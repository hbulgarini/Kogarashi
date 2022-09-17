# same prime number with circom finite field
p = 21888242871839275222246405745257275088548364400416034343698204186575808495617
F = GF(p)

class ElGamal:
    def __init__(self):
        self.F = F
        self.g = F.multiplicative_generator()

    def randomness(self):
        return self.F.random_element()

    def private_key(self):
        return self.randomness()

    def public_key(self, private_key):
        return self.g^private_key

    @staticmethod
    def shared_secret(public_key, private_key):
        return public_key^private_key

    @staticmethod
    def sign_message(message, shared_secret):
        return message * shared_secret

    @staticmethod
    def decrypt(encrypted_message, shared_secret):
        return encrypted_message / shared_secret

    def encrypt(self, message, randomness, public_key):
        return ((self.g^message) * (public_key^randomness), self.g^randomness)

    @staticmethod
    def add(encrypted_a, encrypted_b):
        (encrypted_g_a, encrypted_message_a) = encrypted_a
        (encrypted_g_b, encrypted_message_b) = encrypted_b
        return (encrypted_g_a * encrypted_g_b, encrypted_message_a * encrypted_message_b)

    @staticmethod
    def sub(encrypted_a, encrypted_b):
        (encrypted_g_a, encrypted_message_a) = encrypted_a
        (encrypted_g_b, encrypted_message_b) = encrypted_b
        return (encrypted_g_a / encrypted_g_b, encrypted_message_a / encrypted_message_b)

def confidential_transfer():
    elgamal = ElGamal()

    alice_sk = elgamal.private_key()
    alice_pk = elgamal.public_key(alice_sk)
    bob_sk = elgamal.private_key()
    bob_pk = elgamal.public_key(bob_sk)

    alice_before_balance = 1500
    bob_before_balance = 2300
    transfer_amount = 800
    alice_original_randomness = 789
    bob_original_randomness = 456
    alice_encrypted_before_balance = elgamal.encrypt(alice_before_balance, alice_original_randomness, alice_pk)
    bob_encrypted_before_balance = elgamal.encrypt(bob_before_balance, bob_original_randomness, bob_pk)

    alice_transfer_randomness = 123
    alice_encrypted_transfer_amount = elgamal.encrypt(transfer_amount, alice_transfer_randomness, alice_pk)
    bob_encrypted_transfer_amount = elgamal.encrypt(transfer_amount, alice_transfer_randomness, bob_pk)

    alice_after_encrypted_balance = elgamal.sub(alice_encrypted_before_balance, alice_encrypted_transfer_amount)
    bob_after_encrypted_balance = elgamal.add(bob_encrypted_before_balance, bob_encrypted_transfer_amount)

    alice_after_balance = alice_before_balance - transfer_amount
    bob_after_balance = bob_before_balance + transfer_amount
    alice_random_sum = alice_original_randomness - alice_transfer_randomness
    bob_random_sum = bob_original_randomness + alice_transfer_randomness
    alice_encrypted_after_balance = elgamal.encrypt(alice_after_balance, alice_random_sum, alice_pk)
    bob_encrypted_after_balance = elgamal.encrypt(bob_after_balance, bob_random_sum, bob_pk)

    assert(alice_after_encrypted_balance == alice_encrypted_after_balance)
    assert(bob_after_encrypted_balance == bob_encrypted_after_balance)

    (alice_left_encrypted_balance, alice_right_encrypted_balance) = alice_encrypted_before_balance
    (alice_left_encrypted_transfer_amount, alice_right_encrypted_transfer_amount) = alice_encrypted_transfer_amount
    (bob_left_encrypted_transfer_amount, bob_right_encrypted_transfer_amount) = bob_encrypted_transfer_amount

    contents = {
        "alice_public_key": str(alice_pk),
        "bob_public_key": str(bob_pk),
        "alice_left_encrypted_balance": str(alice_left_encrypted_balance),
        "alice_right_encrypted_balance": str(alice_right_encrypted_balance),
        "alice_left_encrypted_transfer_amount": str(alice_left_encrypted_transfer_amount),
        "alice_right_encrypted_transfer_amount": str(alice_right_encrypted_transfer_amount),
        "bob_left_encrypted_transfer_amount": str(bob_left_encrypted_transfer_amount),
        "bob_right_encrypted_transfer_amount": str(bob_right_encrypted_transfer_amount),
        "generator": str(elgamal.g),
        "alice_private_key": str(alice_sk),
        "transfer_amount_b": str(transfer_amount),
        "alice_after_balance": str(alice_after_balance),
        "randomness": str(alice_transfer_randomness)
    }
    open(f"inputs/confidential_transfer_input.json", "w", encoding='utf-8').write(str(contents).replace("'", '"') + '\n')

confidential_transfer()

def additive_homomorphic():
    elgamal = ElGamal()

    alice_sk = elgamal.private_key()
    alice_pk = elgamal.public_key(alice_sk)

    value = 1500
    value_prime = 2300
    random = 123
    random_prime = 456

    alice_encrypted_value = elgamal.encrypt(value, random, alice_pk)
    alice_encrypted_value_prime = elgamal.encrypt(value_prime, random_prime, alice_pk)
    encrypted_sum = elgamal.add(alice_encrypted_value, alice_encrypted_value_prime)

    value_sum = value + value_prime
    random_sum = random + random_prime
    sum_encrypted = elgamal.encrypt(value_sum, random_sum, alice_pk)

    assert(encrypted_sum == sum_encrypted)

additive_homomorphic()

def dh_key_exchange():
    elgamal = ElGamal()

    alice_sk = elgamal.private_key()
    alice_pk = elgamal.public_key(alice_sk)

    bob_sk = elgamal.private_key()
    bob_pk = elgamal.public_key(bob_sk)

    alice_shared_secret = elgamal.shared_secret(bob_pk, alice_sk)
    alice_secret_message = elgamal.randomness()
    alice_encrypted_message = elgamal.sign_message(alice_secret_message, alice_shared_secret)

    bob_shared_secret = elgamal.shared_secret(alice_pk, bob_sk)
    bob_decrypted_message = elgamal.decrypt(alice_encrypted_message, bob_shared_secret)

    assert(alice_shared_secret == bob_shared_secret)
    assert(alice_secret_message == bob_decrypted_message)

dh_key_exchange()