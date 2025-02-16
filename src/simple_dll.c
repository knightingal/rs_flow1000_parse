#include <stdio.h>
#include <stdlib.h>
#include <string.h>
int simple_dll_function() {
  printf("call simple_dll_function\n");
  return 1;
}

struct rust_object {
  int a;
  int b;
};

struct char_arr_object {
  char* a;
  int b;
};



int simple_dll_function_with_param(struct rust_object* p_rust_object) {
  printf("p_rust_object: %p\n", p_rust_object);
  int a = p_rust_object->a;
  printf("a: %d\n", a);

  int b = p_rust_object->b;
  printf("b: %d\n", b);
  printf("simple_dll_function_with_param \n");
  p_rust_object->b = 20;
  return 1;
}

struct rust_object* simple_dll_function_return_struct() {
  struct rust_object* ro = malloc(sizeof(struct rust_object));
  ro->a = 1;
  ro->b = 10;
  return ro;
}

struct char_arr_object* simple_dll_function_return_char_arr() {
  struct char_arr_object* ro = malloc(sizeof(struct char_arr_object));
  ro->a = "hello";
  ro->b = 10;
  return ro;
}

const char* simple_dll_function_return_heap_point() {
  const char* hp = malloc(10);
  memset(hp, 1, 10);
  return hp;
}