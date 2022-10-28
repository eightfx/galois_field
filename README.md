
# Table of Contents

1.  [finite fields](#orgedaa53a)
2.  [What makes it different from other libraries?](#orga5394ac)
    1.  [Pros:](#org4012cdf)
    2.  [Cons:](#org5343e9f)
3.  [Usage](#orgaa5a15b)
4.  [Examples](#orgb8b597b)
    1.  [Prime Field](#orga3f7a8a)
    2.  [Galois Field](#org32fb5c8)
    3.  [Polynomial over F<sub>p</sub>](#org0bcebbf)
    4.  [Polynomial over GF(p<sup>n</sup>)](#orgf09a81e)
    5.  [Matrix over FiniteField](#orgf1f67ef)


<a id="orgedaa53a"></a>

# finite fields

A Rust library for operations on finite field, featuring:

-   Sum, difference, product, and quotient of elements F<sub>p</sub>
-   Sum, difference, product, and quotient of elements GF(p<sup>n</sup>)
-   Obtaining the primitive polynomial
-   Sum, difference, product, quotient, and remainder of polynomial over F<sub>p</sub>
-   Sum, difference, product, quotient, and remainder of polynomial over GF(p<sup>n</sup>)
-   Sum, product of Matrix F<sub>p</sub>
-   Sum, product of Matrix GF(p<sup>n</sup>)
-   The sweep method (or Gaussian elimination) of matrices on finite bodies (F<sub>p</sub>, GF(p<sup>n</sup>)) is also available.


<a id="orga5394ac"></a>

# What makes it different from other libraries?


<a id="org4012cdf"></a>

## Pros:

-   Can be calculated with F<sub>p</sub>, GF(p<sup>n</sup>) for any prime and any multiplier, not limited to a char 2.
-   Can freely compute six types of element: prime field, Galois field, polynomial of prime field, polynomial of Galois field, matrix of prime field, and matrix of Galois field.
-   Each can be calculated with +-\*/, so you can write natural code.
-   Matrix operations on finite field (F<sub>p</sub>, GF(p<sup>n</sup>)) can also be performed.
-   The sweep method can be available.


<a id="org5343e9f"></a>

## Cons:

-   It takes longer than other libraries because it is not optimized for each character.


<a id="orgaa5a15b"></a>

# Usage

Add this to your Cargo.toml:

    [dependencies]
    galois_field = "0.1.10"


<a id="orgb8b597b"></a>

# Examples


<a id="orga3f7a8a"></a>

## Prime Field

    use galois_field::*;
    
    let char: u32 = 5;
    let x:FiniteField = FiniteField{
    	char: char,
    	element:Element::PrimeField{element:0} // 0 in F_5
    };
    let y:FiniteField = FiniteField{
    	char: char,
    	element:Element::PrimeField{element:1} // 1 in F_5
    };
    println!("x + y = {:?}", (x.clone() + y.clone()).element); // ->1
    println!("x - y = {:?}", (x.clone() - y.clone()).element); // -> 4
    println!("x * y = {:?}", (x.clone() * y.clone()).element); // -> 0
    println!("x / y = {:?}", (x.clone() / y.clone()).element); // -> 0


<a id="org32fb5c8"></a>

## Galois Field

    use galois_field::*;
    
    fn main(){
    	// consider GF(2^4)
    	let char: u32 = 2;
    	let n = 4;
    	let primitive_polynomial = Polynomial::get_primitive_polynomial(char, n);
    	let x:FiniteField = FiniteField{
     		char: char,
     		element:Element::GaloisField{element:vec![0,1],primitive_polynomial:primitive_polynomial.clone()} // i.e. [0,1] = x -> 2 over GF(2^4)
    	};
    	let y:FiniteField = FiniteField{
     		char: char,
     		element:Element::GaloisField{element:vec![0,0,1,1],primitive_polynomial:primitive_polynomial.clone()} // i.e. [0,0,1,1] = x^3 + x^2 -> 12 over GF(2^4)
    	};
    	println!("x + y = {:?}", (x.clone() + y.clone()).element);
    	println!("x - y = {:?}", (x.clone() - y.clone()).element);
    	println!("x * y = {:?}", (x.clone() * y.clone()).element);
    	println!("x / y = {:?}", (x.clone() / y.clone()).element);
    
    }


<a id="org0bcebbf"></a>

## Polynomial over F<sub>p</sub>

    use galois_field::*;
    
    fn main() {
    	// character
        let char: u32 = 2;
    
    	let element0:FiniteField = FiniteField{
    		char: char,
    		element:Element::PrimeField{element:0} // 0 in F_5
    	};
    	let element1:FiniteField = FiniteField{
    		char: char,
    		element:Element::PrimeField{element:1} // 1 in F_5
    	};
    
    
    	let f: Polynomial = Polynomial {
            coef: vec![element1.clone(),element0.clone(),element0.clone(),element0.clone(),element1.clone()]
    	};
        let g: Polynomial = Polynomial {
    		coef: vec![element1.clone(),element0.clone(),element0.clone(),element1.clone(),element1.clone()]
        };
        println!("f + g = {:?}", (f.clone()+g.clone()).coef);
    	println!("f - g = {:?}", (f.clone()-g.clone()).coef);
    	println!("f * g = {:?}", (f.clone()*g.clone()).coef);
    	println!("f / g = {:?}", (f.clone()/g.clone()).coef);
    	println!("f % g = {:?}", (f.clone()%g.clone()).coef);
    	
    }


<a id="orgf09a81e"></a>

## Polynomial over GF(p<sup>n</sup>)

Same as above


<a id="orgf1f67ef"></a>

## Matrix over FiniteField

    use galois_field::*;
    
    let char = 3;
    let element0: FiniteField = FiniteField {
        char: char,
        element: Element::PrimeField { element: 0 },
    };
    let element1: FiniteField = FiniteField {
        char: char,
        element: Element::PrimeField { element: 1 },
    };
    let element2: FiniteField = FiniteField {
        char: char,
        element: Element::PrimeField { element: 2 },
    };
    
    
    let mut matrix_element:Vec<Vec<FiniteField>> = vec![
        vec![element0.clone(),element1.clone(), element0.clone()],
        vec![element2.clone(),element2.clone(), element1.clone()],
        vec![element1.clone(),element0.clone(), element1.clone()]
    ];
    let mut matrix = Matrix{
        element: matrix_element,
    };
    
    println!("m+m = {:?}", m.clone()+m.clone());
    println!("m*m = {:?}", m.clone()*m.clone());
    
    let mut sweep_matrix = m.sweep_method();
    println!("{:?}", sweep_matrix);

