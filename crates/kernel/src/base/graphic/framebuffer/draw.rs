use {
	super::FrameBuffer,
	crate::base::graphic::{
		color::{ColorRpr, PixelFormat},
		position::Coordinal,
	},
	poison_girl_no_std_error::{
		GraphicError, PoisonGirlB, X, Y, poison_girl_err,
	},
};

pub trait DisplayDraw
{
	/// The result type for drawing operations
	type Output = PoisonGirlB<(),>;

	/// Draws a single pixel at the specified coordinate with the given color
	///
	/// # Arguments
	///
	/// * `coord` - The coordinate where the pixel should be drawn
	/// * `color` - The color representation for the pixel
	///
	/// # Returns
	///
	/// * `Ok(())` - If the pixel was successfully drawn
	/// * `Err(GraphicError)` - If the coordinate is invalid or drawing fails
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// let coord = Coord::new(100, 50);
	/// let color = Rgb::new(255, 0, 0); // Red pixel
	/// framebuffer.put_pixel(&coord, &color)?;
	/// ```
	fn put_pixel(
		&self,
		coord: &impl Coordinal,
		color: &impl ColorRpr,
	) -> Self::Output;

	/// Fills a rectangular area with the specified color
	///
	/// This method fills all pixels within the rectangle defined by the
	/// top-left and bottom-right coordinates (inclusive) with the given color.
	///
	/// # Arguments
	///
	/// * `left_top` - The top-left corner coordinate of the rectangle
	/// * `right_bottom` - The bottom-right corner coordinate of the rectangle
	/// * `color` - The color to fill the rectangle with
	///
	/// # Returns
	///
	/// * `Ok(())` - If the rectangle was successfully filled
	/// * `Err(GraphicError::InvalidCoordinate)` - If the coordinates are
	///   invalid
	///
	/// # Coordinate Requirements
	///
	/// The coordinates must satisfy the following conditions:
	/// - `left_top.x < right_bottom.x && left_top.y < right_bottom.y`
	/// - `right_bottom.x <= framebuffer.width && right_bottom.y <=
	///   framebuffer.height`
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// let top_left = Coord::new(10, 10);
	/// let bottom_right = Coord::new(50, 30);
	/// let color = Rgb::new(0, 255, 0); // Green rectangle
	/// framebuffer.fill_rectangle(&top_left, &bottom_right, &color)?;
	/// ```
	fn fill_rectangle(
		&self,
		left_top: &impl Coordinal,
		right_bottom: &impl Coordinal,
		color: &impl ColorRpr,
	) -> Self::Output;

	/// Draws the outline of a rectangle with the specified color
	///
	/// This method draws only the border of the rectangle, leaving the interior
	/// unchanged. The outline is drawn as a single-pixel-wide border.
	///
	/// # Arguments
	///
	/// * `left_top` - The top-left corner coordinate of the rectangle
	/// * `right_bottom` - The bottom-right corner coordinate of the rectangle
	/// * `color` - The color for the rectangle outline
	///
	/// # Returns
	///
	/// * `Ok(())` - If the outline was successfully drawn
	/// * `Err(GraphicError::InvalidCoordinate)` - If the coordinates are
	///   invalid
	///
	/// # Coordinate Requirements
	///
	/// Same requirements as `fill_rectangle()`.
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// let top_left = Coord::new(20, 20);
	/// let bottom_right = Coord::new(80, 60);
	/// let color = Rgb::new(0, 0, 255); // Blue outline
	/// framebuffer.outline_rectangle(&top_left, &bottom_right, &color)?;
	/// ```
	fn outline_rectangle(
		&self,
		left_top: &impl Coordinal,
		right_bottom: &impl Coordinal,
		color: &impl ColorRpr,
	) -> Self::Output;
}

impl<P: PixelFormat,> DisplayDraw for FrameBuffer<P,>
{
	/// Draws a single pixel at the specified coordinate
	///
	/// This implementation writes the color data directly to the framebuffer
	/// memory at the calculated position. It converts the color to the
	/// appropriate pixel format and writes the RGB components.
	///
	/// # Arguments
	///
	/// * `coord` - The coordinate where the pixel should be drawn
	/// * `color` - The color representation to draw
	///
	/// # Returns
	///
	/// * `Ok(())` - If the pixel was successfully drawn
	/// * `Err(GraphicError)` - If an error occurs during drawing
	///
	/// # Implementation Details
	///
	/// 1. Calculates the byte position using the coordinate
	/// 2. Gets a mutable slice to the pixel memory (3 bytes for RGB)
	/// 3. Converts the color using the pixel format's color representation
	/// 4. Writes the RGB components to memory
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// use poison_girl_kernel::base::graphic::position::Coord;
	/// use poison_girl_kernel::base::graphic::color::Rgb;
	///
	/// let coord = Coord::new(100, 50);
	/// let red_color = Rgb::new(255, 0, 0);
	/// framebuffer.put_pixel(&coord, &red_color)?;
	/// ```
	fn put_pixel(
		&self,
		coord: &impl Coordinal,
		color: &impl ColorRpr,
	) -> Self::Output
	{
		let pos = self.pos(coord,);
		let pxl = self.slice_mut(pos, 3,);
		let color = self.drawer.try_color_repr(color,)?;
		pxl[0] = color[0];
		pxl[1] = color[1];
		pxl[2] = color[2];

		X((),)
	}

	/// Fills a rectangular area with the specified color
	///
	/// This implementation validates the coordinates and then iterates through
	/// all pixels within the rectangle, setting each one to the specified
	/// color. The color conversion is performed once before the loop for
	/// efficiency.
	///
	/// # Arguments
	///
	/// * `left_top` - The top-left corner of the rectangle
	/// * `right_bottom` - The bottom-right corner of the rectangle
	/// * `color` - The color to fill the rectangle with
	///
	/// # Returns
	///
	/// * `Ok(())` - If the rectangle was successfully filled
	/// * `Err(GraphicError::InvalidCoordinate)` - If coordinates are invalid
	///
	/// # Coordinate Validation
	///
	/// The method validates that:
	/// - `left_top.x() <= right_bottom.x()`
	/// - `left_top.y() <= right_bottom.y()`
	/// - `right_bottom.x() < self.width`
	/// - `right_bottom.y() < self.height`
	///
	/// # Performance Optimization
	///
	/// The color is converted to the pixel format representation once before
	/// the drawing loop to avoid repeated conversions for each pixel.
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// let top_left = Coord::new(10, 10);
	/// let bottom_right = Coord::new(50, 30);
	/// let blue_color = Rgb::new(0, 0, 255);
	/// framebuffer.fill_rectangle(&top_left, &bottom_right, &blue_color)?;
	/// ```
	fn fill_rectangle(
		&self,
		left_top: &impl Coordinal,
		right_bottom: &impl Coordinal,
		color: &impl ColorRpr,
	) -> Self::Output
	{
		// Validate coordinate bounds
		if left_top.x() > right_bottom.x()
			|| left_top.y() > right_bottom.y()
			|| right_bottom.x() > self.width
			|| right_bottom.y() > self.height
		{
			return Y(poison_girl_err!(GraphicError::InvalidCoordinate),);
		}

		// Convert color once for performance optimization
		// This reduces pixel format determination to just once per rectangle
		let color = self.drawer.try_color_repr(color,)?;
		let mut coord = (left_top.x(), left_top.y(),);

		// Fill rectangle row by row
		for _ in left_top.y()..=right_bottom.y() {
			for _ in left_top.x()..=right_bottom.x() {
				let pos = self.pos(&coord,);
				let pxl = self.slice_mut(pos, 3,);
				pxl[0] = color[0];
				pxl[1] = color[1];
				pxl[2] = color[2];
				coord.0 += 1;
			}
			coord.1 += 1;
			coord.0 = left_top.x();
		}

		X((),)
	}

	/// Draws the outline of a rectangle with the specified color
	///
	/// This implementation draws a single-pixel-wide border around the
	/// rectangle defined by the coordinates. It draws four lines: top, right,
	/// bottom, and left.
	///
	/// # Arguments
	///
	/// * `left_top` - The top-left corner of the rectangle
	/// * `right_bottom` - The bottom-right corner of the rectangle
	/// * `color` - The color for the rectangle outline
	///
	/// # Returns
	///
	/// * `Ok(())` - If the outline was successfully drawn
	/// * `Err(GraphicError::InvalidCoordinate)` - If coordinates are invalid
	///
	/// # Drawing Algorithm
	///
	/// The outline is drawn in four phases:
	/// 1. **Top line**: From left_top to (right_bottom.x, left_top.y)
	/// 2. **Right line**: From (right_bottom.x, left_top.y) to right_bottom
	/// 3. **Bottom line**: From right_bottom to (left_top.x, right_bottom.y)
	/// 4. **Left line**: From (left_top.x, right_bottom.y) to left_top
	///
	/// # Performance Optimization
	///
	/// Like `fill_rectangle`, the color conversion is performed once before
	/// drawing to avoid repeated conversions.
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// let top_left = Coord::new(20, 20);
	/// let bottom_right = Coord::new(80, 60);
	/// let green_color = Rgb::new(0, 255, 0);
	/// framebuffer.outline_rectangle(&top_left, &bottom_right, &green_color)?;
	/// ```
	fn outline_rectangle(
		&self,
		left_top: &impl Coordinal,
		right_bottom: &impl Coordinal,
		color: &impl ColorRpr,
	) -> Self::Output
	{
		// Validate coordinate bounds
		if left_top.x() > right_bottom.x()
			|| left_top.y() > right_bottom.y()
			|| right_bottom.x() > self.width
			|| right_bottom.y() > self.height
		{
			return Y(poison_girl_err!(GraphicError::InvalidCoordinate),);
		}

		let width = right_bottom.x() - left_top.x() - 1;
		let height = right_bottom.y() - left_top.y() - 1;

		// Convert color once for performance
		let color = self.drawer.try_color_repr(color,)?;
		let mut coord = (left_top.x(), left_top.y(),);

		// Draw top horizontal line
		for _ in 0..width {
			let pos = self.pos(&coord,);
			let pxl = self.slice_mut(pos, 3,);
			pxl[0] = color[0];
			pxl[1] = color[1];
			pxl[2] = color[2];
			coord.0 += 1;
		}

		// Draw right vertical line
		for _ in 0..height {
			let pos = self.pos(&coord,);
			let pxl = self.slice_mut(pos, 3,);
			pxl[0] = color[0];
			pxl[1] = color[1];
			pxl[2] = color[2];
			coord.1 += 1;
		}

		// Draw bottom horizontal line
		for _ in 0..width {
			let pos = self.pos(&coord,);
			let pxl = self.slice_mut(pos, 3,);
			pxl[0] = color[0];
			pxl[1] = color[1];
			pxl[2] = color[2];
			coord.0 -= 1;
		}

		// Draw left vertical line
		for _ in 0..height {
			let pos = self.pos(&coord,);
			let pxl = self.slice_mut(pos, 3,);
			pxl[0] = color[0];
			pxl[1] = color[1];
			pxl[2] = color[2];
			coord.1 -= 1;
		}

		X((),)
	}
}
