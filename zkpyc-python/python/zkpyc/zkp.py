import os
from zkpyc.types import _set_modulus
from zkpyc.input_gen import prepare_prover_inputs, prepare_verifier_inputs, process_includes, represent_object
from zkpyc.bindings import compiler, backend

class ZKP:
    _instance = None
    modulus = None
    field = None # this is a temporary solution for type checking in externally called functions

    def __new__(cls, modulus=None, id=0, backend=None, module='__main__'):
        if cls._instance is None:
            field = _set_modulus(modulus)
            cls._instance = super(ZKP, cls).__new__(cls)
            compiler.init(str(field.modulus))
            cls.modulus = field.modulus
            cls.field = field
            cls.id = id
            cls.backend = backend
            cls.module = module
        else:
            raise RuntimeError("Only one instance of the ZKP class is allowed.")
        return cls._instance

    def compile(self, func, includes=None, global_vars=None, local_vars=None):
        # Get the function implementation and name
        func_impl = represent_object(func, current_module=self.module, is_entry_fct=True)
        func_name = func.__name__

        # Get the processed objects
        if includes is None:
            obj_impl = ""
        else:
            obj_impl = process_includes(includes, self.field, self.module, global_vars, local_vars)

        # Concatenate the function definition and processed objects
        code = f"{obj_impl}{func_impl}"
        # print(code)
        return compiler.compile(func_name, code, self.id, f"<{self.module}>")

    def prepare_proof(self, func, *args, **kwargs):
        argument_names = func.__code__.co_varnames[:func.__code__.co_argcount]
        argument_types = func.__annotations__
        lisp_code = prepare_prover_inputs(
            argument_names,
            argument_types,
            self.modulus,
            self.field,
            *args,
            **kwargs
        )

        return compiler.setup_proof(func.__name__, lisp_code, self.id, f"<{self.module}>")

    def prepare_verification(self, func, *args, return_value=None, **kwargs):
        if return_value is None:
            raise ValueError("Missing return value for verification.")

        argument_names = func.__code__.co_varnames[:func.__code__.co_argcount]
        argument_types = func.__annotations__
        return_type = argument_types.get('return', None)

        lisp_code = prepare_verifier_inputs(
            argument_names,
            argument_types,
            return_type,
            self.modulus,
            return_value,
            self.field,
            *args,
            **kwargs,
        )

        return compiler.setup_verification(func.__name__, lisp_code, self.id, f"<{self.module}>")

    def generate_crs(self, func):
        f_name = func.__name__
        header_file = 'cache_id_{}/zkif_export/header_{}_{}.zkif'.format(self.id, f"<{self.module}>", f_name)
        constraints_file = 'cache_id_{}/zkif_export/constraints_{}_{}.zkif'.format(self.id, f"<{self.module}>", f_name)

        with open(header_file, 'rb') as file:
            circuit = file.read()
        with open(constraints_file, 'rb') as file:
            constraints = file.read()

        backend.setup(circuit, constraints, f_name, self.id, f"<{self.module}>", self.backend)

        # finally return the bytestring of the crs
        crs_file = 'cache_id_{}/zkp_params_and_proofs/{}_{}_key.dat'.format(self.id, f"<{self.module}>", f_name)
        with open(crs_file, 'rb') as file:
            crs = file.read()
        return crs

    def store_crs(self, func, crs_bytes):
        f_name = func.__name__
        crs_folder = './cache_id_{}/zkp_params_and_proofs'.format(self.id)
        crs_file = crs_folder + '/{}_{}_key.dat'.format(f"<{self.module}>", f_name)
        if not os.path.exists(crs_folder):
            os.makedirs(crs_folder)
        with open(crs_file, 'wb') as file:
            file.write(crs_bytes)

    def store_proof(self, func, proof_bytes):
        f_name = func.__name__
        proof_folder = './cache_id_{}/zkp_params_and_proofs'.format(self.id)
        proof_file = proof_folder + '/{}_{}_proof.dat'.format(f"<{self.module}>", f_name)
        if not os.path.exists(proof_folder):
            os.makedirs(proof_folder)
        with open(proof_file, 'wb') as file:
            file.write(proof_bytes)

    def run_prover(self, func):
        f_name = func.__name__
        header_file = 'cache_id_{}/zkif_export/header_{}_{}.zkif'.format(self.id, f"<{self.module}>", f_name)
        witness_file = 'cache_id_{}/zkif_export/witness_{}_{}.zkif'.format(self.id, f"<{self.module}>", f_name)
        constraints_file = 'cache_id_{}/zkif_export/constraints_{}_{}.zkif'.format(self.id, f"<{self.module}>", f_name)

        with open(header_file, 'rb') as file:
            circuit = file.read()
        with open(witness_file, 'rb') as file:
            witness = file.read()
        with open(constraints_file, 'rb') as file:
            constraints = file.read()

        backend.prove(circuit, witness, constraints, f_name, self.id, f"<{self.module}>", self.backend)

    def run_verifier(self, func):
        f_name = func.__name__
        header_file = 'cache_id_{}/zkif_export/header_{}_{}.zkif'.format(self.id, f"<{self.module}>", f_name)
        constraints_file = 'cache_id_{}/zkif_export/constraints_{}_{}.zkif'.format(self.id, f"<{self.module}>", f_name)

        with open(header_file, 'rb') as file:
            circuit = file.read()
        with open(constraints_file, 'rb') as file:
            constraints = file.read()

        return backend.verify(circuit, constraints, f_name, self.id, f"<{self.module}>", self.backend)

    def prove(self, func, *args, **kwargs):
        # first we obtain correct circuit and witness zkif files from python inputs
        # then we run the backend's prover
        self.prepare_proof(func, *args, **kwargs)
        self.run_prover(func)

        # finally return the bytestring of proof
        f_name = func.__name__
        proof_file = 'cache_id_{}/zkp_params_and_proofs/{}_{}_proof.dat'.format(self.id, f"<{self.module}>", f_name)
        with open(proof_file, 'rb') as file:
            proof = file.read()
        return proof

    def verify(self, func, *args, return_value=None, **kwargs):
        # first we obtain correct circuit zkif files from python inputs
        # then we run the backend's verifier
        self.prepare_verification(func, *args, return_value=return_value, **kwargs)
        return self.run_verifier(func)

    def cleanup(self):
        compiler.cleanup(self.id)
