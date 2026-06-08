// Optional C++ convenience layer over the Wickra C ABI (`wickra.h`).
//
// The C ABI hands out raw handles that must be released exactly once with the
// matching `wickra_<ind>_free`. `wickra::Handle` wraps that in a move-only RAII
// owner so the free happens automatically at scope exit:
//
//     #include "wickra.hpp"
//
//     wickra::Handle<Sma, wickra_sma_free> sma(wickra_sma_new(14));
//     if (sma) {
//         double v = wickra_sma_update(sma.get(), 42.0);  // NaN during warmup
//     }
//     // sma is freed here
//
// This is header-only and adds no runtime cost beyond the C calls themselves.

#ifndef WICKRA_HPP
#define WICKRA_HPP

#include "wickra.h"

#include <utility>

namespace wickra {

/// Move-only RAII owner of a Wickra handle. `T` is the opaque indicator type and
/// `Free` its `wickra_<ind>_free` function.
template <typename T, void (*Free)(T *)>
class Handle {
public:
    explicit Handle(T *ptr) noexcept : ptr_(ptr) {}

    ~Handle() {
        if (ptr_ != nullptr) {
            Free(ptr_);
        }
    }

    Handle(const Handle &) = delete;
    Handle &operator=(const Handle &) = delete;

    Handle(Handle &&other) noexcept : ptr_(std::exchange(other.ptr_, nullptr)) {}

    Handle &operator=(Handle &&other) noexcept {
        if (this != &other) {
            if (ptr_ != nullptr) {
                Free(ptr_);
            }
            ptr_ = std::exchange(other.ptr_, nullptr);
        }
        return *this;
    }

    /// The raw handle, for passing to the `wickra_<ind>_*` functions.
    T *get() const noexcept { return ptr_; }

    /// True if the handle is non-null (construction succeeded).
    explicit operator bool() const noexcept { return ptr_ != nullptr; }

private:
    T *ptr_;
};

}  // namespace wickra

#endif  // WICKRA_HPP
