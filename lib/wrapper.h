/**
 * @file wrapper.h
 * @brief This file serves as a wrapper to include essential Zsh header files.
 *
 * It aggregates standard and configuration headers required for Zsh module development,
 * as well as component-specific headers that provide access to various Zsh internal
 * functionalities and data structures. This wrapper simplifies the inclusion process
 * for `bindgen` in the Rust build script.
 */
#ifndef WRAPPER_H
#define WRAPPER_H

/* Standard and configuration headers */
#include "config.h"
#include "zsh.mdh"
#include "zsh_system.h"
#include "zshpaths.h"

/* Component specific headers */
#include "hashtable.h"
#include "patchlevel.h"
#include "prototypes.h"
#include "sigcount.h"
#include "signals.h"
#include "version.h"
#include "wcwidth9.h"
#include "zshcurses.h"
#include "zshterm.h"
#include "zshxmods.h"
#include "ztype.h"

#endif /* WRAPPER_H */