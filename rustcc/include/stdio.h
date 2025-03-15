/**
 * stdio.h - Minimal implementation for testing
 * 
 * This is a simplified version of stdio.h for testing the RustCC preprocessor
 */

#ifndef _STDIO_H
#define _STDIO_H

/* Standard I/O functions */
int printf();
int fprintf();
int sprintf();
int snprintf();

/* File operations */
typedef void FILE;

/* Standard streams */
#define stdin  0
#define stdout 1
#define stderr 2

/* EOF definition */
#define EOF (-1)

#endif /* _STDIO_H */ 