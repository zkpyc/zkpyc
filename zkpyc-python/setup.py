import os
import shutil
import sys
import importlib.util
from setuptools import setup, find_packages
from setuptools.command.build_py import build_py
from setuptools.command.build_ext import build_ext
from setuptools_rust import RustExtension


class CpyStdlibAndBuildPy(build_py):
    def run(self):
        
        # Build the rust extension module
        build_ext_cmd = self.get_finalized_command('build_ext')
        build_ext_cmd.run()
        ext_path = build_ext_cmd.ext_path
        ext_path = ext_path.rsplit('.', 2)[0] + ".abi3.so" # rename due to abi3 feature
        ext_dir = os.path.dirname(ext_path)
        sys.path.insert(0, ext_dir)

        if not os.path.isfile(ext_path):
            raise Exception(f"Rust extension not found at {ext_path}")

        # Import extension module and call get_stdlib_path to get temporary artifact path
        spec = importlib.util.spec_from_file_location('zkpyc.bindings._rust', ext_path)
        zkpyc_bindings = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(zkpyc_bindings)
        stdlib_path = zkpyc_bindings.get_stdlib_path()

        # Move stdlib into zkpyc package
        dest_stdlib_dir = os.path.join("python", "zkpyc", "stdlib")
        if not os.path.exists(dest_stdlib_dir):
            os.makedirs(dest_stdlib_dir)
        shutil.copytree(stdlib_path, dest_stdlib_dir, dirs_exist_ok=True)

        # Update package_dir info
        self.distribution.package_dir.update({
            "zkpyc.stdlib": "python/zkpyc/stdlib"
        })
        self.distribution.packages.append("zkpyc.stdlib")

        # Proceed with the build
        super().run()


class BuildExt(build_ext):
    def run(self):
        super().run()
        
        # Store path to the built Rust extension
        self.ext_path = self.get_ext_fullpath("zkpyc.bindings._rust")


setup(
    name="zkpyc",
    version="0.1.0",
    packages=find_packages(where="python"),
    package_dir={
        "": "python",
    },
    rust_extensions=[
        RustExtension("zkpyc.bindings._rust", binding="pyo3"),
    ],
    include_package_data=True,
    zip_safe=False,
    cmdclass={
        'build_ext': BuildExt,
        'build_py': CpyStdlibAndBuildPy,
    },
)