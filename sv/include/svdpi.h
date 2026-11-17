/*
 * svdpi.h for epitome-sv: the C side of SystemVerilog's DPI, as IEEE
 * 1800-2023 Annex H defines it. Libraries built against it run in
 * epitome-sv (and, the types and functions being the standard's, in other
 * simulators).
 *
 * The functions are defined by the simulator executable; on macOS, link a
 * DPI library with `-undefined dynamic_lookup` so that it finds them there.
 * The deprecated SV3.1a functions of H.14 are not provided.
 */

#ifndef EPITOME_SVDPI_H
#define EPITOME_SVDPI_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* bit and logic scalars (H.10.1.1) */
typedef uint8_t svScalar;
typedef svScalar svBit;
typedef svScalar svLogic;

#define sv_0 0
#define sv_1 1
#define sv_z 2
#define sv_x 3

/* Packed values in canonical form (H.7.7, H.10.1.2): 32 bits a word,
   least significant word first. For four-state values, aval/bval are 0/0
   for 0, 1/0 for 1, 0/1 for z and 1/1 for x. Bits past the width are
   undefined. */
#ifndef VPI_VECVAL
#define VPI_VECVAL
typedef struct t_vpi_vecval {
    uint32_t aval;
    uint32_t bval;
} s_vpi_vecval, *p_vpi_vecval;
#endif

typedef s_vpi_vecval svLogicVecVal;
typedef uint32_t svBitVecVal;

/* The number of words for a packed value of WIDTH bits. */
#define SV_PACKED_DATA_NELEMS(WIDTH) (((WIDTH) + 31) >> 5)

/* The low N bits of a word, and a word's low N bits read unsigned or
   sign-extended. */
#define SV_MASK(N) (~(-1 << (N)))
#define SV_GET_UNSIGNED_BITS(VALUE, N) ((N) == 32 ? (VALUE) : ((VALUE) & SV_MASK(N)))
#define SV_GET_SIGNED_BITS(VALUE, N) \
    ((N) == 32 ? (VALUE) : (((VALUE) & (1 << (N))) ? ((VALUE) | ~SV_MASK(N)) : ((VALUE) & SV_MASK(N))))

/* Handles (H.10.1.3). */
typedef void *svScope;
typedef void *svOpenArrayHandle;

/* Time (H.13), the same as VPI's s_vpi_time. */
#ifndef VPI_TIME
#define VPI_TIME
typedef struct t_vpi_time {
    int32_t type;
    uint32_t high, low;
    double real;
} s_vpi_time, *p_vpi_time;
#define vpiScaledRealTime 1
#define vpiSimTime 2
#define vpiSuppressTime 3
#endif

typedef s_vpi_time svTimeVal;

#define sv_scaled_real_time vpiScaledRealTime
#define sv_sim_time vpiSimTime

/* The canonical representation this implementation provides. */
const char *svDpiVersion(void);

/* Bits and part-selects of canonical values, indexed [n-1:0] (H.11.5).
   A part-select is at most 32 bits wide. */
svBit svGetBitselBit(const svBitVecVal *s, int i);
svLogic svGetBitselLogic(const svLogicVecVal *s, int i);
void svPutBitselBit(svBitVecVal *d, int i, svBit s);
void svPutBitselLogic(svLogicVecVal *d, int i, svLogic s);
void svGetPartselBit(svBitVecVal *d, const svBitVecVal *s, int i, int w);
void svGetPartselLogic(svLogicVecVal *d, const svLogicVecVal *s, int i, int w);
void svPutPartselBit(svBitVecVal *d, const svBitVecVal s, int i, int w);
void svPutPartselLogic(svLogicVecVal *d, const svLogicVecVal s, int i, int w);

/* Open arrays: their dimensions (H.12.2), 0 for the packed one. */
int svLeft(const svOpenArrayHandle h, int d);
int svRight(const svOpenArrayHandle h, int d);
int svLow(const svOpenArrayHandle h, int d);
int svHigh(const svOpenArrayHandle h, int d);
int svIncrement(const svOpenArrayHandle h, int d);
int svSize(const svOpenArrayHandle h, int d);
int svDimensions(const svOpenArrayHandle h);

/* Their storage (H.12.4), indexed by the actual argument's ranges. */
void *svGetArrayPtr(const svOpenArrayHandle h);
int svSizeOfArray(const svOpenArrayHandle h);
void *svGetArrElemPtr(const svOpenArrayHandle h, int indx1, ...);
void *svGetArrElemPtr1(const svOpenArrayHandle h, int indx1);
void *svGetArrElemPtr2(const svOpenArrayHandle h, int indx1, int indx2);
void *svGetArrElemPtr3(const svOpenArrayHandle h, int indx1, int indx2, int indx3);

/* Packed elements, copied to and from canonical form (H.12.5). */
void svPutBitArrElemVecVal(const svOpenArrayHandle d, const svBitVecVal *s, int indx1, ...);
void svPutBitArrElem1VecVal(const svOpenArrayHandle d, const svBitVecVal *s, int indx1);
void svPutBitArrElem2VecVal(const svOpenArrayHandle d, const svBitVecVal *s, int indx1, int indx2);
void svPutBitArrElem3VecVal(const svOpenArrayHandle d, const svBitVecVal *s, int indx1, int indx2, int indx3);
void svPutLogicArrElemVecVal(const svOpenArrayHandle d, const svLogicVecVal *s, int indx1, ...);
void svPutLogicArrElem1VecVal(const svOpenArrayHandle d, const svLogicVecVal *s, int indx1);
void svPutLogicArrElem2VecVal(const svOpenArrayHandle d, const svLogicVecVal *s, int indx1, int indx2);
void svPutLogicArrElem3VecVal(const svOpenArrayHandle d, const svLogicVecVal *s, int indx1, int indx2, int indx3);
void svGetBitArrElemVecVal(svBitVecVal *d, const svOpenArrayHandle s, int indx1, ...);
void svGetBitArrElem1VecVal(svBitVecVal *d, const svOpenArrayHandle s, int indx1);
void svGetBitArrElem2VecVal(svBitVecVal *d, const svOpenArrayHandle s, int indx1, int indx2);
void svGetBitArrElem3VecVal(svBitVecVal *d, const svOpenArrayHandle s, int indx1, int indx2, int indx3);
void svGetLogicArrElemVecVal(svLogicVecVal *d, const svOpenArrayHandle s, int indx1, ...);
void svGetLogicArrElem1VecVal(svLogicVecVal *d, const svOpenArrayHandle s, int indx1);
void svGetLogicArrElem2VecVal(svLogicVecVal *d, const svOpenArrayHandle s, int indx1, int indx2);
void svGetLogicArrElem3VecVal(svLogicVecVal *d, const svOpenArrayHandle s, int indx1, int indx2, int indx3);

/* Scalar elements (H.12.6). */
svBit svGetBitArrElem(const svOpenArrayHandle s, int indx1, ...);
svBit svGetBitArrElem1(const svOpenArrayHandle s, int indx1);
svBit svGetBitArrElem2(const svOpenArrayHandle s, int indx1, int indx2);
svBit svGetBitArrElem3(const svOpenArrayHandle s, int indx1, int indx2, int indx3);
svLogic svGetLogicArrElem(const svOpenArrayHandle s, int indx1, ...);
svLogic svGetLogicArrElem1(const svOpenArrayHandle s, int indx1);
svLogic svGetLogicArrElem2(const svOpenArrayHandle s, int indx1, int indx2);
svLogic svGetLogicArrElem3(const svOpenArrayHandle s, int indx1, int indx2, int indx3);
void svPutLogicArrElem(const svOpenArrayHandle d, svLogic value, int indx1, ...);
void svPutLogicArrElem1(const svOpenArrayHandle d, svLogic value, int indx1);
void svPutLogicArrElem2(const svOpenArrayHandle d, svLogic value, int indx1, int indx2);
void svPutLogicArrElem3(const svOpenArrayHandle d, svLogic value, int indx1, int indx2, int indx3);
void svPutBitArrElem(const svOpenArrayHandle d, svBit value, int indx1, ...);
void svPutBitArrElem1(const svOpenArrayHandle d, svBit value, int indx1);
void svPutBitArrElem2(const svOpenArrayHandle d, svBit value, int indx1, int indx2);
void svPutBitArrElem3(const svOpenArrayHandle d, svBit value, int indx1, int indx2, int indx3);

/* Context (H.9.3): the calling chain's scope, setting it before calling
   an export, scopes by name, and data kept per scope and key. */
svScope svGetScope(void);
svScope svSetScope(const svScope scope);
const char *svGetNameFromScope(const svScope scope);
svScope svGetScopeFromName(const char *scopeName);
int svPutUserData(const svScope scope, void *userKey, void *userData);
void *svGetUserData(const svScope scope, void *userKey);
int svGetCallerInfo(const char **fileName, int *lineNumber);

/* Disabling (35.9). */
int svIsDisabledState(void);
void svAckDisabledState(void);

/* Time (H.13); a NULL scope means the simulation's time unit. */
int svGetTime(const svScope scope, svTimeVal *time);
int svGetTimeUnit(const svScope scope, int32_t *time_unit);
int svGetTimePrecision(const svScope scope, int32_t *time_precision);

#ifdef __cplusplus
}
#endif

#endif
