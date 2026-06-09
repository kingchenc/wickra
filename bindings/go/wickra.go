// Package wickra provides idiomatic Go bindings for the Wickra
// technical-analysis library over its C ABI hub.
//
// Each indicator is an opaque-handle type with a New<Indicator> constructor and
// Update/Batch/Reset/Close methods. Handles are freed by Close and, as a
// backstop, by a finalizer; call Close explicitly to release native memory
// promptly. The binding links against the prebuilt Wickra C ABI library
// (libwickra.so/.dylib or wickra.dll) staged under ./lib — see the package
// README for how to provision it.
package wickra

/*
#cgo CFLAGS: -I${SRCDIR}/../c/include
#cgo linux LDFLAGS: -L${SRCDIR}/lib -lwickra -Wl,-rpath,${SRCDIR}/lib
#cgo darwin LDFLAGS: -L${SRCDIR}/lib -lwickra -Wl,-rpath,${SRCDIR}/lib
#cgo windows LDFLAGS: -L${SRCDIR}/lib -l:wickra.dll
#include "wickra.h"
*/
import "C"

import "errors"

// ErrInvalidParams is returned by a New<Indicator> constructor when the native
// constructor rejects the supplied parameters (for example a zero period).
var ErrInvalidParams = errors.New("wickra: invalid indicator parameters")
